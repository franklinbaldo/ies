//! CLI Web de controle de prazo: leitura, prévia e confirmação local testável.
//! O binário de distribuição continua recusando escrita antes da rede.
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    consulta_do_controlador, derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, leitura, ler_cookie, obter_tela};
use crate::processo_obter::EnvioDoProcesso;
use crate::processo_obter::{ProcessoAberto, SessaoDoProcesso, abrir_para_escrever};
use crate::processo_prazo::{
    EstadoDoPrazo, PedidoDePrazo, TelaDoPrazo, campos_do_pedido, interpretar_pedido, ler_tela,
};
use crate::visao_do_controle::{EnvioDoControle as _, EnvioHttps};
use crate::{Cli, Domain, Processo, ProcessoAcao, processo_concluir_cli};
use serde_json::{Value, json};

mod exclusao;

/// Recusas e efeito incerto do controle de prazo; nenhuma mensagem paralela.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDoPrazo {
    Pedido(#[from] crate::processo_prazo::ErroPrazo),
    /// A barra do processo não oferece controle de prazo.
    ControleNaoOferecidoNaBarra,
    /// Pedido, charset ou origem não permitem derivar a submissão.
    ConfirmacaoSemFormularioSeguro,
    /// A submissão foi recusada antes de o POST sair.
    SubmissaoRecusada(ErroSubmissaoFormulario),
    /// O POST saiu e a releitura não confirmou: não repetir.
    NaoConfirmado(Option<ErroSubmissaoFormulario>),
    /// O processo não oferece a leitura independente do prazo.
    LeituraIndependenteNaoOferecida,
    /// O Controle da unidade não publicou um link de lista de prazos aceito.
    LinkDaListaDePrazos(#[source] exclusao::ErroLinkDaLista),
    /// A lista não traz o processo: não há controle a excluir.
    ProcessoForaDaListaDePrazos,
    /// A unidade da sessão não foi publicada pela tela do processo.
    UnidadeDaSessaoNaoPublicada,
    /// A lista de prazos fora da forma observada ou sem um único controle
    /// do processo.
    ListaDePrazos(#[source] exclusao::ErroListaDePrazos),
    /// O POST saiu e a lista relida não pôde ser lida: efeito incerto.
    ListaRelidaIlegivel(#[source] exclusao::ErroListaDePrazos),
    /// A tela do prazo, relida depois do POST, não foi obtida.
    TelaRelidaNaoObtida(#[source] crate::processo_concluir::escrita::FalhaDaTela),
    /// O processo, reaberto depois do POST, não foi aberto para conferência.
    ProcessoRelidoNaoAbriu(#[source] crate::processo_obter::ErroProcessoObter),
    /// O prazo relido depois do POST não tem forma legível.
    PrazoRelidoIlegivel(#[source] crate::processo_prazo::ErroPrazo),
    /// A lista não publicou uma ação de exclusão derivável.
    AcaoDeExclusao(#[source] exclusao::ErroAcaoDeExclusao),
    /// O link do prazo, relido depois do POST, não pôde ser lido.
    LinkDePrazoRelidoIlegivel(#[source] crate::arvore_processo::ErroLinkPrazo),
}

pub(crate) fn tentar_prazo(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Prazo {
                numero,
                ate,
                dias,
                excluir,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = numero.clone();
    // O clap já conferiu formatos, exclusividade e que --confirmar tem o que gravar.
    let pedido = match interpretar_pedido(ate.as_deref(), *dias, *excluir) {
        Ok(pedido) => pedido,
        Err(erro) => return Some(Err(FalhaDoPrazo::from(erro).into())),
    };
    Some(processo_concluir_cli::com_sessao(cli, |sessao, indice| {
        // Aprovada em treinamento (10/11/2026): grava também em produção.
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoPrazo,
                sessao.origem,
            )
        {
            return Err(causa.into());
        }
        if pedido == PedidoDePrazo::Excluir {
            exclusao::executar(cli, &nup, *confirmar, sessao, indice)
        } else {
            previsualizar(cli, &nup, &pedido, *confirmar, sessao, indice)
        }
    }))
}

fn previsualizar(
    cli: &Cli,
    nup: &NumeroProcesso,
    pedido: &PedidoDePrazo,
    confirmar: bool,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
) -> anyhow::Result<()> {
    let mut envio = EnvioHttps;
    let mut rede = Contador::novo(&mut envio);
    let abertura = leitura(nup, cli.unidade.as_deref(), true);
    let (aberto, acoes) = abrir_para_escrever(sessao, &abertura, indice, &mut rede)?;
    let link = acoes
        .prazo?
        .ok_or(FalhaDoPrazo::ControleNaoOferecidoNaBarra)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) = obter_tela(&mut rede, &cookie, &link)?;
    let tela = ler_tela(&html, &aberto.arvore.processo.id).map_err(FalhaDoPrazo::from)?;
    let tela = if confirmar {
        confirmar_prazo(
            sessao,
            &cookie,
            &link,
            &html,
            &mime,
            pedido,
            &aberto.arvore.processo.id,
            &mut rede,
        )?
    } else {
        tela
    };
    publicar_previa(cli, nup, pedido, &aberto, &tela, rede.feitas(), confirmar);
    Ok(())
}

/// Publica a prévia lida, sem escrever: estado atual, pedido e campos.
fn publicar_previa(
    cli: &Cli,
    nup: &NumeroProcesso,
    pedido: &PedidoDePrazo,
    aberto: &ProcessoAberto,
    tela: &TelaDoPrazo,
    requisicoes: u8,
    confirmado: bool,
) {
    let atual = match &tela.estado {
        EstadoDoPrazo::SemPrazo => json!({"tipo": "sem_prazo"}),
        EstadoDoPrazo::Data(data) => json!({"tipo": "data", "data": data}),
        EstadoDoPrazo::Dias(dias) => json!({"tipo": "dias", "dias": dias}),
        EstadoDoPrazo::Concluido => json!({"tipo": "concluido"}),
    };
    let solicitado = match pedido {
        PedidoDePrazo::Ate(data) => json!({"ate": data}),
        PedidoDePrazo::Dias(dias) => json!({"dias": dias}),
        PedidoDePrazo::Ler | PedidoDePrazo::Excluir => Value::Null,
    };
    let campos: Vec<_> = campos_do_pedido(pedido)
        .into_iter()
        .map(|(nome, _)| nome)
        .collect();
    let resultado = json!({
        "nup": nup.as_str(),
        "id_procedimento": aberto.arvore.processo.id,
        "unidade": aberto.unidade,
        "origem": "web",
        "atual": atual,
        "valor_do_checkbox_de_dias_uteis": tela.valor_dos_uteis,
        "solicitado": solicitado,
        "campos_da_previa": campos,
        "confirmado": confirmado,
        "requisicoes": requisicoes,
    });
    if cli.json {
        println!("{resultado}");
    } else {
        println!("Processo {} — prazo atual: {}", nup.as_str(), atual);
        if confirmado {
            println!("Prazo confirmado por releitura: {solicitado}.");
        } else if !solicitado.is_null() {
            println!("Prévia: {solicitado}. Confirmação disponível somente no simulador local.");
        }
        println!("Requisições: {requisicoes}");
    }
}

const DEFINIR_PRAZO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ControlePrazoDefinir,
    formulario: "frmControlePrazoCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmDefinirControlePrazo"),
    campos_ocultos_exigidos: &[
        "hdnInfraTipoPagina",
        "hdnIdControlePrazo",
        "hdnIdProcedimentos",
    ],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["rdoPrazo", "txtPrazo", "txtDias"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::ControleDePrazo,
    },
};

/// O 302 deve nomear uma única ação, o resultado e o mesmo processo.
fn aceite_confirmacao(local: &str, url_post: &str, id: &str) -> bool {
    let Some(consulta) = consulta_do_controlador(local, url_post) else {
        return false;
    };
    let mut acoes = Vec::new();
    let mut resultados = Vec::new();
    let mut processos = Vec::new();
    for parte in consulta.split('&') {
        let Some((chave, valor)) = parte.split_once('=') else {
            return false;
        };
        match chave {
            "acao" => acoes.push(valor),
            "resultado" => resultados.push(valor),
            "id_procedimento" => processos.push(valor),
            _ => {}
        }
    }
    acoes == ["controle_prazo_definir"]
        && resultados == ["1"]
        && (processos.is_empty() || processos == [id])
}

/// Apenas o primeiro POST tem efeito; erro depois do envio é incerto.
#[allow(clippy::too_many_arguments)]
fn confirmar_prazo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    cookie: &str,
    link: &str,
    html: &str,
    mime: &str,
    pedido: &PedidoDePrazo,
    id: &str,
    rede: &mut Contador<'_, V>,
) -> Result<TelaDoPrazo, FalhaDoPrazo> {
    if matches!(pedido, PedidoDePrazo::Ler | PedidoDePrazo::Excluir) {
        return Err(FalhaDoPrazo::ConfirmacaoSemFormularioSeguro);
    }
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(FalhaDoPrazo::ConfirmacaoSemFormularioSeguro)?;
    let codificacao =
        codificacao_da_tela(mime).ok_or(FalhaDoPrazo::ConfirmacaoSemFormularioSeguro)?;
    let servida = TelaServida {
        url: link,
        html,
        codificacao,
    };
    let campos = campos_do_pedido(pedido);
    let valores: Vec<_> = campos
        .iter()
        .map(|(nome, valor)| (*nome, valor.as_str()))
        .collect();
    let submissao = derivar_submissao(&DEFINIR_PRAZO, &instalacao, &servida, "", &valores)
        .map_err(FalhaDoPrazo::SubmissaoRecusada)?;
    let mut enviou = false;
    let resultado = enviar_submissao_via(submissao, sessao.cookie, |post, c| {
        enviou = true;
        rede.post(post, c)
    })
    .map_err(|erro| {
        if enviou {
            FalhaDoPrazo::NaoConfirmado(Some(erro))
        } else {
            FalhaDoPrazo::SubmissaoRecusada(erro)
        }
    })?;
    match resultado {
        RespostaDoEnvio::Redirecionamento(redir)
            if redir
                .localizacao()
                .is_some_and(|local| aceite_confirmacao(local, redir.url_do_envio(), id)) => {}
        // A `UnB` responde 200 com o formulário redesenhado (cassette
        // processo-prazo-unb, interações 12 e 19; live de 10/11/2026): a
        // prova fica com a releitura independente abaixo.
        RespostaDoEnvio::Html(_) => {}
        RespostaDoEnvio::Redirecionamento(_) => return Err(FalhaDoPrazo::NaoConfirmado(None)),
    }
    let (nova, _) = obter_tela(rede, cookie, link).map_err(FalhaDoPrazo::TelaRelidaNaoObtida)?;
    let verificada = ler_tela(&nova, id).map_err(FalhaDoPrazo::PrazoRelidoIlegivel)?;
    let hoje = (time::OffsetDateTime::now_utc().date() - time::OffsetDateTime::UNIX_EPOCH.date())
        .whole_days();
    if !crate::processo_prazo::releitura_confirma(pedido, &verificada.estado, hoje) {
        return Err(FalhaDoPrazo::NaoConfirmado(None));
    }
    Ok(verificada)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processo_prazo::ErroPrazo;

    #[test]
    fn tipos_de_entrada_distintos_nao_sao_tratados_como_escrita() {
        assert_eq!(
            interpretar_pedido(None, None, false),
            Ok(PedidoDePrazo::Ler)
        );
        assert_eq!(
            interpretar_pedido(None, None, true),
            Ok(PedidoDePrazo::Excluir)
        );
        assert_eq!(
            interpretar_pedido(Some("31/02/2027"), None, false),
            Err(ErroPrazo::DataInvalida)
        );
    }
}
