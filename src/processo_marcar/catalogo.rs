//! BDD-255-01: atribuição ausente exige catálogo publicado no formulário da unidade.
//! Prévia não modifica processo e confirmação só será permitida após aceite causal.
use super::{
    Decisao, ErroMarcar, FalhaDaMarcacao, Operacao, Rotulo, decidir, recusa,
    releitura_de_marcadores, texto_relido,
};
use crate::Cli;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{
    CampoDeTexto, ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, Redirecionamento302, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    consulta_do_controlador, derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    abrir_para_escrever,
};
use crate::transporte::decodificar_resposta;
use crate::visao_do_controle::EnvioDoControle as _;
use serde_json::json;

const ADICIONAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::AndamentoMarcadorCadastrar,
    formulario: "frmAndamentoMarcadorCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmSalvar"),
    campos_ocultos_exigidos: &["hdnInfraTipoPagina", "hdnIdMarcador", "hdnIdProtocolo"],
    campos_ocultos_reescritos: &["hdnIdMarcador"],
    // O ddslick substitui o select no envio; não se inventa selMarcador no POST.
    campos_usuario: &["txaTexto"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::MarcadoresNoProcesso,
    },
};

/// Extrai o link do botão `btnAdicionar` servido, sem executar JavaScript.
/// O link só vale depois de `conferir_endereco_relativo` (ação, processo e
/// assinatura); a forma do botão não é conferida aqui.
pub(super) fn link_de_adicionar(
    html: &crate::tela_html::HtmlServido,
) -> Result<String, ErroMarcar> {
    let dom = crate::tela_html::arvore(html);
    dom.elementos("button")
        .filter(|tag| tag.atributo("id") == Some("btnAdicionar"))
        .find_map(|tag| {
            let onclick = tag.atributo("onclick").map(str::to_owned)?;
            let relativo = onclick
                .strip_prefix("location.href='")?
                .strip_suffix('\'')?;
            Some(relativo.to_owned())
        })
        .ok_or(ErroMarcar::DestinoDoCadastroAusente)
}

fn destino_cadastro<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    instalacao: &Instalacao,
    id: &str,
) -> anyhow::Result<String> {
    let resposta = rede.get_sem_seguir(link, cookie)?;
    let destino = if resposta.resposta.status_code == 302 {
        let redir = resposta
            .localizacao
            .ok_or_else(|| recusa(ErroMarcar::DestinoDoCadastroAusente, rede.feitas()))?;
        let prefixo = format!("{}/", instalacao.url_base());
        redir.strip_prefix(&prefixo).unwrap_or(&redir).to_owned()
    } else {
        let html = decodificar_resposta(resposta.resposta)?;
        link_de_adicionar(&html).map_err(|causa| recusa(causa, rede.feitas()))?
    };
    conferir_endereco_relativo(instalacao, &destino, ADICIONAR.acao, id).map_err(|recusado| {
        let causa = match recusado {
            EnderecoRecusado::NaoReconhecido => ErroMarcar::EnderecoDoCadastroNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => ErroMarcar::EnderecoDoCadastroDeOutroProcesso,
        };
        recusa(causa, rede.feitas())
    })
}

/// A identidade do catálogo vem de option[value=id] no select da unidade,
/// não do marcador aplicado ao processo nem de uma lista sem origem: o id
/// escrito precisa estar oferecido com o nome pedido.
pub(super) fn rotulo_oferecido(
    html: &crate::tela_html::HtmlServido,
    rotulo: &Rotulo,
) -> Result<(), ErroMarcar> {
    let dom = crate::tela_html::arvore(html);
    let nomes: Vec<String> = dom
        .elementos("select")
        .filter(|tag| tag.atributo("id") == Some("selMarcador"))
        .flat_map(crate::tela_html::Elemento::descendentes)
        .filter(|tag| tag.nome() == "option")
        .filter(|tag| tag.atributo("value") == Some(rotulo.id.as_str()))
        .map(|tag| tag.texto())
        .collect();
    if nomes.is_empty() {
        return Err(ErroMarcar::OpcaoDoMarcadorNaoOferecida);
    }
    if nomes.iter().any(|nome| *nome != rotulo.nome) {
        return Err(ErroMarcar::NomeDoMarcadorDiferenteDoCatalogo);
    }
    Ok(())
}

fn submissao_cadastro<V: EnvioDoProcesso>(
    (origem, cookie): (&str, &str),
    rede: &mut Contador<'_, V>,
    link: &str,
    id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    texto: Option<&str>,
) -> anyhow::Result<crate::formulario_web::SubmissaoDerivada> {
    let instalacao = Instalacao::interpretar(origem)
        .ok_or_else(|| recusa(ErroMarcar::OrigemNaoInterpretada, rede.feitas()))?;
    let url = destino_cadastro(rede, cookie, link, &instalacao, id)?;
    let (html, mime) = match obter_tela(rede, cookie, &url) {
        Ok(tela) => tela,
        Err(FalhaDaTela::Transporte(erro)) => return Err(erro.into()),
        Err(FalhaDaTela::Redirecionada | FalhaDaTela::SessaoEncerrada(_)) => {
            return Err(recusa(
                ErroMarcar::FormularioDeCadastroNaoLido,
                rede.feitas(),
            ));
        }
    };
    let codificacao = codificacao_da_tela(&mime)
        .ok_or_else(|| recusa(ErroMarcar::CharsetDoFormularioNaoSuportado, rede.feitas()))?;
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao,
    };
    rotulo_oferecido(&html, rotulo).map_err(|causa| recusa(causa, rede.feitas()))?;
    derivar_submissao(
        &ADICIONAR,
        &instalacao,
        &tela,
        Some(unidade),
        &[
            ("hdnIdMarcador", &rotulo.id),
            ("txaTexto", CampoDeTexto::de(texto).enviado()),
        ],
    )
    .map_err(Into::into)
}

fn saida_previa(
    cli: &Cli,
    nup: &NumeroProcesso,
    destino: (&str, &str),
    rotulo: &Rotulo,
    texto: Option<&str>,
    nada_a_fazer: bool,
    requisicoes: u8,
) {
    let (id, unidade) = destino;
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(), "id_procedimento": id, "unidade": unidade,
                "marcador": format!("{}#{}", rotulo.id, rotulo.nome),
                "texto": texto, "acao": "atribuir", "origem": "web",
                "confirmado": false, "alterado": false,
                "nada_a_fazer": nada_a_fazer,
                "retirada_possivel": false, "requisicoes": requisicoes,
            })
        );
    } else if nada_a_fazer {
        println!(
            "Processo {}: nada a fazer com {} (atribuir). Requisições: {}.",
            nup.as_str(),
            rotulo.id,
            requisicoes
        );
    } else {
        println!(
            "Prévia: atribuir marcador {}#{} ao processo {} (unidade {}, texto: {:?}). Requisições: {}. Esta prévia não alterou o processo. Para tentar a atribuição, use --confirmar em treinamento; o catálogo e o formulário precisam estar conferidos.",
            rotulo.id,
            rotulo.nome,
            nup.as_str(),
            unidade,
            CampoDeTexto::de(texto).enviado(),
            requisicoes
        );
    }
}

/// Atribuição ausente só produz prévia se a opção exata estiver na tela
/// independente de cadastro da unidade, incluindo validação do form servido.
pub(crate) fn executar_atribuicao<V: EnvioDoProcesso>(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    (rotulo, texto, confirmar): (&Rotulo, Option<&str>, bool),
    envio: &mut V,
) -> anyhow::Result<()> {
    let mut rede = Contador::novo(envio);
    let pedido = PedidoDoProcesso {
        nup,
        unidade: cli.unidade.as_deref(),
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: true,
        unidade_exigida: true,
        leitura_das_pastas: crate::processo_obter::LeituraDasPastas::Estrita,
    };
    let (aberto, acoes) = abrir_para_escrever(sessao, &pedido, indice, &mut rede)?;
    let Some(unidade) = aberto.unidade.as_deref() else {
        return Err(recusa(
            ErroMarcar::UnidadeDaSessaoNaoPublicada,
            rede.feitas(),
        ));
    };
    let Marcadores::Lidos(ref lidos) = aberto.marcadores else {
        return Err(recusa(ErroMarcar::MarcadoresNaoLidos, rede.feitas()));
    };
    let nada = match decidir(lidos, rotulo, Operacao::Atribuir) {
        Ok(Decisao::NadaAFazer) => true,
        Ok(Decisao::PreviaDeAtribuicao) => false,
        Ok(Decisao::PreviaDeRetirada) => unreachable!(),
        Err(e) => return Err(recusa(e, rede.feitas())),
    };
    if nada {
        saida_previa(
            cli,
            nup,
            (&aberto.arvore.processo.id, unidade),
            rotulo,
            texto,
            true,
            rede.feitas(),
        );
        return Ok(());
    }
    let link = acoes
        .marcadores?
        .ok_or_else(|| recusa(ErroMarcar::LinkDeMarcadoresNaoPublicado, rede.feitas()))?;
    let submissao = submissao_cadastro(
        (sessao.origem, &acoes.cookie),
        &mut rede,
        link.as_str(),
        &aberto.arvore.processo.id,
        unidade,
        rotulo,
        texto,
    )?;
    if !confirmar {
        saida_previa(
            cli,
            nup,
            (&aberto.arvore.processo.id, unidade),
            rotulo,
            texto,
            false,
            rede.feitas(),
        );
        return Ok(());
    }
    finalizar_atribuicao(
        cli,
        (sessao, indice, &pedido),
        nup,
        (&aberto.arvore.processo.id, unidade),
        (rotulo, texto),
        &mut rede,
        submissao,
    )
}

/// Após uma única tentativa, a resposta causal e a releitura são provas separadas.
fn finalizar_atribuicao<V: EnvioDoProcesso>(
    cli: &Cli,
    contexto: (&SessaoDoProcesso<'_>, &IndiceLocal, &PedidoDoProcesso<'_>),
    nup: &NumeroProcesso,
    destino: (&str, &str),
    marcador: (&Rotulo, Option<&str>),
    rede: &mut Contador<'_, V>,
    submissao: crate::formulario_web::SubmissaoDerivada,
) -> anyhow::Result<()> {
    let (sessao, indice, pedido) = contexto;
    let (id, unidade) = destino;
    let (rotulo, texto) = marcador;
    let mut tentativa = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |submissao, cookie| {
        tentativa = true;
        rede.post(submissao, cookie)
    });
    if !tentativa {
        return Err(match resposta {
            Err(erro) => erro.into(),
            Ok(_) => recusa(ErroMarcar::SubmissaoDoMarcadorNaoEnviada, rede.feitas()),
        });
    }
    let aceito = matches!(
        resposta,
        Ok(RespostaDoEnvio::Redirecionamento(ref redir))
            if aceite_da_atribuicao(redir, id)
    );
    // Segunda leitura independente: não confiar no próprio POST nem repetir.
    let presente =
        releitura_de_marcadores(sessao, indice, pedido, rede, (id, unidade)).map(|lidos| {
            lidos.itens.iter().any(|m| {
                m.id == rotulo.id
                    && m.nome == rotulo.nome
                    && texto_relido(m.texto.as_deref(), CampoDeTexto::de(texto))
            })
        });
    if !aceito || !matches!(presente, Ok(true)) {
        return Err(FalhaDaMarcacao::AtribuicaoNaoConfirmada {
            requisicoes: rede.feitas(),
            presente_relido: presente,
        }
        .into());
    }
    saida_confirmada(cli, nup, id, unidade, rotulo, texto, rede.feitas());
    Ok(())
}

pub(super) fn aceite_da_atribuicao(redir: &Redirecionamento302, id: &str) -> bool {
    let Some(consulta) = redir
        .localizacao()
        .and_then(|url| consulta_do_controlador(url, redir.url_do_envio()))
    else {
        return false;
    };
    let Ok(consulta) = ConsultaDoControlador::ler(consulta) else {
        return false;
    };
    consulta.acao() == Ok(AcaoDoSei::AndamentoMarcadorGerenciar)
        && consulta.acao_de_origem() == Ok(AcaoDoSei::AndamentoMarcadorCadastrar)
        && consulta.unico("id_procedimento") == Ok(id)
}

fn saida_confirmada(
    cli: &Cli,
    nup: &NumeroProcesso,
    id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    texto: Option<&str>,
    requisicoes: u8,
) {
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(), "id_procedimento": id, "unidade": unidade,
                "marcador": format!("{}#{}", rotulo.id, rotulo.nome),
                "texto": texto, "acao": "atribuir", "origem": "web",
                "confirmado": true, "alterado": true,
                "nada_a_fazer": false, "requisicoes": requisicoes,
            })
        );
    } else {
        println!(
            "Atribuição confirmada: marcador {}#{} no processo {} (unidade {}). Requisições: {}.",
            rotulo.id,
            rotulo.nome,
            nup.as_str(),
            unidade,
            requisicoes
        );
    }
}
