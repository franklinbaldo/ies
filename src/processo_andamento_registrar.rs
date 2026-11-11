//! #259: registro Web de andamento, com prévia e verificação independente.
//! Casos: Todos src/todos/andamento.py @323eb01.
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::ErroAndamentoNaBarra;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    consulta_do_controlador, derivar_submissao, enviar_submissao_via, localizar_formulario,
};
use crate::historico_processo::AndamentosLidos;
use crate::html_atributos::valor_do_atributo;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, ler_cookie, obter_tela};
use crate::processo_obter::{
    Andamentos, EnvioDoProcesso, ErroProcessoObter, PedidoDeAndamentos, PedidoDoProcesso,
    SessaoDoProcesso, abrir_para_escrever, obter_processo,
};
use crate::visao_do_controle::EnvioDoControle as _;

const FORM: &str = "frmAtividadeListar";
const TEXTO: &str = "txaDescricao";
const BOTAO: &str = "sbmSalvar";
const OCULTOS: [&str; 2] = ["hdnIdAtividade", "hdnIdProtocolo"];

const REGISTRAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoAtualizarAndamento,
    formulario: FORM,
    envio: EnvioDoFormulario::BotaoDeEnvio(BOTAO),
    campos_ocultos_exigidos: &OCULTOS,
    campos_ocultos_reescritos: &[],
    campos_usuario: &[TEXTO],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AndamentoNoHistorico,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTela {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    HtmlInvalido(#[source] tl::ParseError),
    FormularioAusenteOuDuplicado(#[source] ErroSubmissaoFormulario),
    ControleAusenteOuDuplicado,
    ProtocoloDiferente,
    CampoInesperado,
}

/// Um formulário, dois ocultos de identidade e um botão submit.
pub(crate) fn conferir_formulario(html: &str, id: &str) -> Result<(), ErroTela> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(ErroTela::HtmlInvalido)?;
    let form = localizar_formulario(&dom, FORM).map_err(ErroTela::FormularioAusenteOuDuplicado)?;
    let mut protocolo = Vec::new();
    let mut atividade = 0;
    let mut descricao = 0;
    let mut salvar = 0;
    for tag in form
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        if !matches!(
            elemento.as_str(),
            "input" | "textarea" | "button" | "select"
        ) {
            continue;
        }
        let attr = |nome| valor_do_atributo(tag, nome).map_err(ErroTela::Atributo);
        let Some(nome) = attr("name")?.filter(|s| !s.is_empty()) else {
            continue;
        };
        let tipo = attr("type")?
            .unwrap_or_else(|| {
                if elemento == "button" {
                    "submit"
                } else {
                    "text"
                }
                .to_owned()
            })
            .to_ascii_lowercase();
        if attr("disabled")?.is_some() {
            continue;
        }
        match (elemento.as_str(), tipo.as_str(), nome.as_str()) {
            ("input", "hidden", "hdnIdProtocolo") => {
                protocolo.push(attr("value")?.unwrap_or_default());
            }
            ("input", "hidden", "hdnIdAtividade") => atividade += 1,
            ("textarea", _, TEXTO) => descricao += 1,
            ("input" | "button", "submit", BOTAO) => salvar += 1,
            ("input", "hidden", _) | ("input" | "button", "button" | "reset", _) => {}
            _ => return Err(ErroTela::CampoInesperado),
        }
    }
    let ([protocolo], 1, 1, 1) = (protocolo.as_slice(), atividade, descricao, salvar) else {
        return Err(ErroTela::ControleAusenteOuDuplicado);
    };
    if protocolo != id {
        return Err(ErroTela::ProtocoloDiferente);
    }
    Ok(())
}

pub(crate) fn texto_valido(texto: &str) -> bool {
    !texto.trim().is_empty() && texto.chars().count() <= 500
}

/// Texto do andamento vazio ou com mais de 500 caracteres.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct TextoDoAndamentoInvalido;

/// `--texto`: o clap recusa o inválido (saída 2) antes da sessão.
pub(crate) fn texto_do_andamento(texto: &str) -> Result<String, TextoDoAndamentoInvalido> {
    if texto_valido(texto) {
        Ok(texto.to_owned())
    } else {
        Err(TextoDoAndamentoInvalido)
    }
}
fn normalizar(texto: &str) -> String {
    texto.split_whitespace().collect::<Vec<_>>().join(" ")
}
/// Novo topo e as linhas anteriores deslocadas, inclusive em caso de texto igual.
pub(crate) fn inserido_uma_vez(
    antes: &AndamentosLidos,
    depois: &AndamentosLidos,
    texto: &str,
) -> bool {
    let Some((novo, cauda)) = depois.itens.split_first() else {
        return false;
    };
    if antes.tipo != depois.tipo
        || depois.total_declarado != antes.total_declarado.saturating_add(1)
        || normalizar(&novo.descricao) != normalizar(texto)
    {
        return false;
    }
    cauda.iter().zip(&antes.itens).all(|(a, b)| a == b)
}
fn localizacao_esperada(local: Option<&str>, origem: &str, id: &str) -> bool {
    let Some(consulta) = local.and_then(|l| consulta_do_controlador(l, origem)) else {
        return false;
    };
    let Ok(consulta) = ConsultaDoControlador::ler(consulta) else {
        return false;
    };
    consulta.acao() == Ok(AcaoDoSei::ProcedimentoConsultarHistorico)
        && consulta.unico("id_procedimento") == Ok(id)
}
pub(crate) struct Resultado {
    pub(crate) numero: String,
    pub(crate) texto: String,
    pub(crate) confirmado: bool,
    pub(crate) requisicoes: u8,
    pub(crate) total_antes: u64,
    pub(crate) total_depois: Option<u64>,
    pub(crate) resposta_inesperada: Option<RespostaInesperada>,
}
/// O POST saiu, mas a resposta não foi o 302 de volta ao processo; a
/// releitura decide. Publicada no JSON de sucesso (`resposta_inesperada`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RespostaInesperada {
    RespostaDoAndamentoNaoEsperada,
    RespostaDoAndamentoRecusada,
}

/// Por que o POST enviado não foi comprovado pela releitura do histórico.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum IncertezaDoAndamento {
    Releitura(#[source] ErroProcessoObter),
    HistoricoNaoRelido,
    AndamentoNaoInseridoUmaVez,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroRegistrarAndamento {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(#[from] CookieIlegivel),
    Abertura(#[from] ErroProcessoObter),
    OrigemNaoAutorizada,
    Barra(#[from] ErroAndamentoNaBarra),
    AcaoNaoOferecida,
    HistoricoNaoPublicado,
    UnidadeNaoPublicada,
    Tela(#[from] FalhaDaTela),
    FormatoDaTela(#[from] ErroTela),
    CharsetNaoSuportado,
    Formulario(#[from] ErroSubmissaoFormulario),
    /// O POST saiu e a releitura não comprovou o andamento: não repita.
    NaoConfirmado {
        requisicoes: u8,
        resposta_inesperada: Option<RespostaInesperada>,
        #[source]
        causa: IncertezaDoAndamento,
    },
}
/// Fluxo injetável, sem retries, sem redirecionamentos seguidos.
pub(crate) fn registrar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    numero: &NumeroProcesso,
    texto: &str,
    confirmar: bool,
    unidade: Option<&str>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Resultado, ErroRegistrarAndamento> {
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(ErroRegistrarAndamento::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let pedido = PedidoDoProcesso {
        nup: numero,
        unidade,
        andamentos: PedidoDeAndamentos::VisaoServida,
        propriedades: false,
        marcadores: false,
        unidade_exigida: true,
        leitura_das_pastas: crate::processo_obter::LeituraDasPastas::Estrita,
    };
    let (aberto, acoes) = abrir_para_escrever(sessao, &pedido, indice, &mut rede)?;
    let endereco = acoes
        .andamento?
        .ok_or(ErroRegistrarAndamento::AcaoNaoOferecida)?;
    let Andamentos::Lidos(antes) = aberto.andamentos else {
        return Err(ErroRegistrarAndamento::HistoricoNaoPublicado);
    };
    let unidade = aberto
        .unidade
        .ok_or(ErroRegistrarAndamento::UnidadeNaoPublicada)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) = obter_tela(&mut rede, &cookie, endereco.as_str())?;
    conferir_formulario(&html, &aberto.arvore.processo.id)?;
    let codificacao =
        codificacao_da_tela(&mime).ok_or(ErroRegistrarAndamento::CharsetNaoSuportado)?;
    let tela = TelaServida {
        url: endereco.as_str(),
        html: &html,
        codificacao,
    };
    let submissao = derivar_submissao(&REGISTRAR, &instalacao, &tela, &unidade, &[(TEXTO, texto)])?;
    let mut resultado = Resultado {
        numero: numero.as_str().to_owned(),
        texto: texto.to_owned(),
        confirmado: confirmar,
        requisicoes: rede.feitas(),
        total_antes: antes.total_declarado,
        total_depois: None,
        resposta_inesperada: None,
    };
    if !confirmar {
        return Ok(resultado);
    }
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |sub, c| {
        enviado = true;
        rede.post(sub, c)
    });
    resultado.resposta_inesperada = match resposta {
        Ok(RespostaDoEnvio::Redirecionamento(r))
            if localizacao_esperada(
                r.localizacao(),
                r.url_do_envio(),
                &aberto.arvore.processo.id,
            ) =>
        {
            None
        }
        Ok(_) => Some(RespostaInesperada::RespostaDoAndamentoNaoEsperada),
        Err(e) if !enviado => return Err(ErroRegistrarAndamento::Formulario(e)),
        Err(_) => Some(RespostaInesperada::RespostaDoAndamentoRecusada),
    };
    // A escrita saiu: mesmo se o POST falhou, jamais repetir.
    let incerto = |rede: &Contador<'_, V>, causa| ErroRegistrarAndamento::NaoConfirmado {
        requisicoes: rede.feitas(),
        resposta_inesperada: resultado.resposta_inesperada,
        causa,
    };
    let depois = match obter_processo(sessao, &pedido, indice, &mut rede) {
        Ok(depois) => depois,
        Err(erro) => return Err(incerto(&rede, IncertezaDoAndamento::Releitura(erro))),
    };
    let Andamentos::Lidos(depois) = depois.andamentos else {
        return Err(incerto(&rede, IncertezaDoAndamento::HistoricoNaoRelido));
    };
    if !inserido_uma_vez(&antes, &depois, texto) {
        return Err(incerto(
            &rede,
            IncertezaDoAndamento::AndamentoNaoInseridoUmaVez,
        ));
    }
    resultado.total_depois = Some(depois.total_declarado);
    resultado.requisicoes = rede.feitas();
    Ok(resultado)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::historico_processo::TipoDoHistorico;
    use crate::processo::Andamento;
    fn linha(texto: &str) -> Andamento {
        Andamento {
            ocorrido_em: "01/11/2026 10:00".into(),
            unidade: Some("A".into()),
            usuario: Some("aluno".into()),
            descricao: texto.into(),
        }
    }
    fn lista(total: u64, textos: &[&str]) -> AndamentosLidos {
        AndamentosLidos {
            tipo: TipoDoHistorico::Resumido,
            total_declarado: total,
            itens: textos.iter().map(|x| linha(x)).collect(),
        }
    }
    // Caso: Todos andamento.py::_inserido_no_topo @323eb01.
    #[test]
    fn texto_igual_exige_nova_linha_e_total_incrementado() {
        let antes = lista(2, &["igual", "antigo"]);
        assert!(inserido_uma_vez(
            &antes,
            &lista(3, &["igual", "igual", "antigo"]),
            "igual"
        ));
        assert!(!inserido_uma_vez(&antes, &antes, "igual"));
        assert!(!inserido_uma_vez(
            &antes,
            &lista(3, &["igual", "outro"]),
            "igual"
        ));
    }
    #[test]
    fn historico_trocado_ou_total_igual_nao_prova_escrita() {
        let antes = lista(10, &["antigo"]);
        let mut depois = lista(11, &["novo", "antigo"]);
        depois.tipo = TipoDoHistorico::Completo;
        assert!(!inserido_uma_vez(&antes, &depois, "novo"));
        assert!(!inserido_uma_vez(
            &antes,
            &lista(10, &["novo", "antigo"]),
            "novo"
        ));
    }
    #[test]
    fn texto_vazio_e_maior_que_500_caracteres_falham() {
        assert!(!texto_valido(" \n\t "));
        assert!(texto_valido(&"á".repeat(500)));
        assert!(!texto_valido(&"á".repeat(501)));
    }

    #[test]
    fn limite_de_500_conta_o_texto_que_sera_enviado_sem_aparar() {
        assert!(!texto_valido(&format!(" {}", "á".repeat(500))));
        assert!(texto_valido(&format!(" {}", "á".repeat(499))));
    }
    fn formulario(id: &str) -> String {
        format!(
            "<form id='frmAtividadeListar' method='post'>\
            <input type='hidden' name='hdnIdProtocolo' value='{id}'>\
            <input type='hidden' name='hdnIdAtividade' value=''>\
            <textarea name='txaDescricao'></textarea>\
            <input type='submit' name='sbmSalvar' value='Salvar'></form>"
        )
    }
    #[test]
    fn formulario_exige_identidade_unica() {
        let html = formulario("123");
        assert_eq!(conferir_formulario(&html, "123"), Ok(()));
        assert_eq!(
            conferir_formulario(&html, "124"),
            Err(ErroTela::ProtocoloDiferente)
        );
        assert!(matches!(
            conferir_formulario(&format!("{html}{html}"), "123"),
            Err(ErroTela::FormularioAusenteOuDuplicado(_))
        ));
    }
    #[test]
    fn campo_inesperado_nao_pode_ser_enviado() {
        let html =
            formulario("123").replace("</form>", "<input name='acao' value='invasor'></form>");
        assert_eq!(
            conferir_formulario(&html, "123"),
            Err(ErroTela::CampoInesperado)
        );
    }
}

#[cfg(test)]
mod acervo;
#[cfg(test)]
#[path = "processo_andamento_registrar/tests_fluxo.rs"]
mod tests_fluxo;
