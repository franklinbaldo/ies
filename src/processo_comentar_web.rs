//! Prévia e confirmação de comentário por links servidos e POST único.
//! O 302 isolado não prova efeito; só releitura completa com id e texto.
use crate::arvore_processo::conferir_endereco_relativo;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, consulta_do_controlador,
    decodificar_atributo, derivar_submissao, enviar_submissao_via,
};
use crate::html_atributos::atributos;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_comentar::conferir_efeito;
use crate::processo_comentarios::{TelaDosComentarios, ler_tela};
use crate::processo_concluir::escrita::{Contador, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso, abrir_para_escrever};
use crate::visao_do_controle::EnvioDoControle as _;

const COMENTAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "comentario_cadastrar",
    formulario: "frmComentarioCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmCadastrarComentario"),
    campos_ocultos_exigidos: &[],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txaDescricao"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::ComentarioNoProcesso,
    },
};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Erro {
    Abertura,
    Barra,
    Cookie,
    Lista,
    LinkNaoPublicado,
    LinkInvalido,
    Cadastro,
    Formulario,
    Charset,
    EfeitoIncerto,
}

pub(crate) struct Previa {
    pub(crate) numero: String,
    pub(crate) id_procedimento: String,
    pub(crate) texto: String,
    pub(crate) requisicoes: u8,
    pub(crate) confirmado: bool,
}

/// Encontra exatamente um link de Novo Comentário publicado no HTML da lista.
/// A URL é conferida pela política canônica do controlador e pela identidade.
fn link_do_cadastro(instalacao: &Instalacao, html: &str, id: &str) -> Result<String, Erro> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| Erro::Lista)?;
    let mut links = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(Erro::Lista)?;
        let href = atributos(tag)
            .and_then(|a| a.valor("href"))
            .map_err(|_| Erro::Lista)?;
        if let Some(href) = href {
            let href = decodificar_atributo(href).ok_or(Erro::Lista)?;
            if href
                .split(['?', '&'])
                .any(|campo| campo == "acao=comentario_cadastrar")
            {
                links.push(href);
            }
        }
    }
    let [unico] = links.as_slice() else {
        return Err(Erro::LinkNaoPublicado);
    };
    conferir_endereco_relativo(instalacao, unico, "comentario_cadastrar", id)
        .map_err(|_| Erro::LinkInvalido)
}

/// Confere a identidade do aceite sem seguir o redirecionamento.
fn id_do_aceite<'a>(local: &'a str, url_post: &str, id_processo: &str) -> Option<&'a str> {
    let consulta = consulta_do_controlador(local, url_post)?;
    let mut acao = None;
    let mut id = None;
    let mut processo = None;
    for campo in consulta.split('&') {
        let (nome, valor) = campo.split_once('=')?;
        match nome {
            "acao" if acao.replace(valor).is_some() => return None,
            "id_comentario" if id.replace(valor).is_some() => return None,
            "id_procedimento" if processo.replace(valor).is_some() => return None,
            "acao" | "id_comentario" | "id_procedimento" => {}
            _ => {}
        }
    }
    (acao == Some("comentario_listar") && processo.is_none_or(|processo| processo == id_processo))
        .then_some(id?)
        .filter(|valor| !valor.is_empty() && valor.bytes().all(|b| b.is_ascii_digit()))
}

/// Uma tentativa de POST e uma releitura da lista; jamais reenvia a escrita.
fn confirmar<V: EnvioDoProcesso>(
    submissao: crate::formulario_web::SubmissaoDerivada,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link_lista: &str,
    id_processo: &str,
    texto: &str,
) -> Result<(), Erro> {
    let mut enviou = false;
    let retorno = enviar_submissao_via(submissao, sessao.cookie, |post, cookie| {
        enviou = true;
        rede.post(post, cookie)
    })
    .map_err(|_| {
        if enviou {
            Erro::EfeitoIncerto
        } else {
            Erro::Formulario
        }
    })?;
    let RespostaDoEnvio::Redirecionamento(redir) = retorno else {
        return Err(Erro::EfeitoIncerto);
    };
    let id_aceito = redir
        .localizacao()
        .and_then(|local| id_do_aceite(local, redir.url_do_envio(), id_processo))
        .ok_or(Erro::EfeitoIncerto)?;
    let (html, _) = obter_tela(rede, cookie, link_lista).map_err(|_| Erro::EfeitoIncerto)?;
    let TelaDosComentarios::Lista(relidos) = ler_tela(&html).map_err(|_| Erro::EfeitoIncerto)?
    else {
        return Err(Erro::EfeitoIncerto);
    };
    conferir_efeito(&relidos, id_aceito, texto).map_err(|_| Erro::EfeitoIncerto)
}

/// Abre o formulário publicado pela lista do processo; grava só se confirmado.
///
/// # Errors
/// Recusa barra, lista, link ou formulário não comprovado.
pub(crate) fn prever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    texto: &str,
    confirmar_escrita: bool,
    unidade: Option<&str>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, Erro> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(Erro::Abertura)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(nup, unidade, false), indice, &mut rede)
            .map_err(|_| Erro::Abertura)?;
    let link = acoes
        .comentarios
        .map_err(|_| Erro::Barra)?
        .ok_or(Erro::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao).map_err(|_| Erro::Cookie)?;
    let (html, _) = obter_tela(&mut rede, &cookie, link.as_str()).map_err(|_| Erro::Lista)?;
    let TelaDosComentarios::Lista(_) = ler_tela(&html).map_err(|_| Erro::Lista)? else {
        return Err(Erro::Lista);
    };
    let id = &aberto.arvore.processo.id;
    let url = link_do_cadastro(&instalacao, &html, id)?;
    let (cadastro, mime) = obter_tela(&mut rede, &cookie, &url).map_err(|_| Erro::Cadastro)?;
    let TelaDosComentarios::CadastroSemComentarios =
        ler_tela(&cadastro).map_err(|_| Erro::Cadastro)?
    else {
        return Err(Erro::Cadastro);
    };
    let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime).ok_or(Erro::Charset)?;
    let tela = TelaServida {
        url: &url,
        html: &cadastro,
        codificacao,
    };
    // Pré-derivação segura do POST, nunca enviada: método, identidade,
    // action assinada e controles precisam estar comprovados.
    let submissao = derivar_submissao(
        &COMENTAR,
        &instalacao,
        &tela,
        aberto.unidade.as_deref().unwrap_or(""),
        &[("txaDescricao", texto)],
    )
    .map_err(|_| Erro::Formulario)?;
    let endereco = submissao
        .url()
        .strip_prefix(&format!("{}/", instalacao.url_base()))
        .ok_or(Erro::Formulario)?;
    conferir_endereco_relativo(&instalacao, endereco, "comentario_cadastrar", id)
        .map_err(|_| Erro::Formulario)?;
    if confirmar_escrita {
        confirmar(submissao, sessao, &mut rede, &cookie, &link, id, texto)?;
    }
    Ok(Previa {
        numero: nup.as_str().to_owned(),
        id_procedimento: id.to_owned(),
        texto: texto.to_owned(),
        requisicoes: rede.feitas(),
        confirmado: confirmar_escrita,
    })
}
