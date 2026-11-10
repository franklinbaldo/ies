//! BDD-255-01: atribuição ausente exige catálogo publicado no formulário da unidade.
//! Prévia não modifica processo e confirmação só será permitida após aceite causal.
use super::{Decisao, ErroMarcar, Operacao, Rotulo, decidir};
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, TelaServida, decodificar_atributo, derivar_submissao,
};
use crate::html_atributos::valor_do_atributo;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    abrir_para_escrever,
};
use crate::transporte::decodificar_resposta;
use crate::{Cli, saida_erro};
use serde_json::json;
use std::process::ExitCode;

const ADICIONAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "andamento_marcador_cadastrar",
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

fn falha(cli: &Cli, erro: ErroMarcar, reqs: u8) -> ExitCode {
    crate::processo_marcar_cli::falha(cli, erro, reqs)
}

/// Extrai link do botão efetivamente servido, sem executar JavaScript.
/// O link retornado só é usado depois da validação de ação, processo e assinatura.
fn link_de_adicionar(html: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let mut links = Vec::new();
    for tag in dom
        .query_selector("button")?
        .filter_map(|h| h.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        if valor_do_atributo(tag, "id").ok()?.as_deref() != Some("btnAdicionar") {
            continue;
        }
        let onclick = valor_do_atributo(tag, "onclick").ok()??;
        let relativo = onclick
            .strip_prefix("location.href='")?
            .strip_suffix('\'')?;
        links.push(decodificar_atributo(relativo)?);
    }
    match links.as_slice() {
        [link] => Some(link.clone()),
        _ => None,
    }
}

fn destino_cadastro<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    instalacao: &Instalacao,
    id: &str,
) -> Option<String> {
    let resposta = rede.get_sem_seguir(link, cookie).ok()?;
    let destino = if resposta.resposta.status_code == 302 {
        let redir = resposta.localizacao?;
        let prefixo = format!("{}/", instalacao.url_base());
        redir.strip_prefix(&prefixo).unwrap_or(&redir).to_owned()
    } else {
        let html = decodificar_resposta(resposta.resposta).ok()?;
        link_de_adicionar(&html)?
    };
    conferir_endereco_relativo(instalacao, &destino, ADICIONAR.acao, id).ok()
}

/// A identidade do catálogo vem de option[value=id] no select da unidade,
/// não do marcador aplicado ao processo nem de uma lista sem origem.
fn rotulo_oferecido(html: &str, rotulo: &Rotulo) -> bool {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return false;
    };
    let parser = dom.parser();
    let mut selects = dom
        .query_selector("select")
        .into_iter()
        .flatten()
        .filter_map(|h| h.get(parser).and_then(tl::Node::as_tag))
        .filter(|tag| {
            valor_do_atributo(tag, "id").ok().flatten().as_deref() == Some("selMarcador")
        });
    let Some(select) = selects.next() else {
        return false;
    };
    if selects.next().is_some() {
        return false;
    }
    let mut encontrados = 0;
    for tag in select
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|tag| tag.name().as_utf8_str() == "option")
    {
        let Ok(Some(id)) = valor_do_atributo(tag, "value") else {
            return false;
        };
        if id == rotulo.id {
            encontrados += 1;
            let nome = tag.inner_text(parser).to_string();
            if decodificar_atributo(&nome).as_deref() != Some(rotulo.nome.as_str()) {
                return false;
            }
        }
    }
    encontrados == 1
}

fn conferir_catalogo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Contador<'_, V>,
    link: &str,
    id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    texto: Option<&str>,
) -> bool {
    let Ok(cookie) = ler_cookie(sessao) else {
        return false;
    };
    let Some(instalacao) = Instalacao::interpretar(sessao.origem) else {
        return false;
    };
    let Some(url) = destino_cadastro(rede, &cookie, link, &instalacao, id) else {
        return false;
    };
    let Ok((html, mime)) = obter_tela(rede, &cookie, &url) else {
        return false;
    };
    let Some(codificacao) = codificacao_da_tela(&mime) else {
        return false;
    };
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao,
    };
    rotulo_oferecido(&html, rotulo)
        && derivar_submissao(
            &ADICIONAR,
            &instalacao,
            &tela,
            unidade,
            &[
                ("hdnIdMarcador", &rotulo.id),
                ("txaTexto", texto.unwrap_or("")),
            ],
        )
        .is_ok()
}

fn saida_previa(
    cli: &Cli,
    nup: &NumeroProcesso,
    id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    texto: Option<&str>,
    nada_a_fazer: bool,
    requisicoes: u8,
) -> ExitCode {
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
            "Prévia: atribuir marcador {}#{} ao processo {} (unidade {}, texto: {:?}). Requisições: {}. Esta prévia não alterou o processo. A confirmação ainda não está habilitada.",
            rotulo.id,
            rotulo.nome,
            nup.as_str(),
            unidade,
            texto.unwrap_or(""),
            requisicoes
        );
    }
    ExitCode::SUCCESS
}

/// Atribuição ausente só produz prévia se a opção exata estiver na tela
/// independente de cadastro da unidade, incluindo validação do form servido.
pub(crate) fn previa_atribuicao<V: EnvioDoProcesso>(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    rotulo: &Rotulo,
    texto: Option<&str>,
    envio: &mut V,
) -> ExitCode {
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
    let (aberto, acoes) = match abrir_para_escrever(sessao, &pedido, indice, &mut rede) {
        Ok(resposta) => resposta,
        Err(erro) => {
            return saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            );
        }
    };
    let Some(unidade) = aberto.unidade.as_deref() else {
        return falha(cli, ErroMarcar::UnidadeDaSessaoNaoPublicada, rede.feitas());
    };
    let Marcadores::Lidos(ref lidos) = aberto.marcadores else {
        return falha(cli, ErroMarcar::MarcadoresNaoLidos, rede.feitas());
    };
    let nada = match decidir(lidos, rotulo, Operacao::Atribuir) {
        Ok(Decisao::NadaAFazer) => true,
        Err(ErroMarcar::CatalogoDaUnidadeNaoObservado) => false,
        Ok(Decisao::PreviaDeRetirada) => unreachable!(),
        Err(e) => return falha(cli, e, rede.feitas()),
    };
    if !nada {
        let Some(link) = acoes.marcadores.ok().flatten() else {
            return falha(
                cli,
                ErroMarcar::CatalogoDaUnidadeNaoObservado,
                rede.feitas(),
            );
        };
        if !conferir_catalogo(
            sessao,
            &mut rede,
            link.as_str(),
            &aberto.arvore.processo.id,
            unidade,
            rotulo,
            texto,
        ) {
            return falha(
                cli,
                ErroMarcar::CatalogoDaUnidadeNaoObservado,
                rede.feitas(),
            );
        }
    }
    saida_previa(
        cli,
        nup,
        &aberto.arvore.processo.id,
        unidade,
        rotulo,
        texto,
        nada,
        rede.feitas(),
    )
}
