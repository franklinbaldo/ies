//! Prévia e confirmação de comentário por links servidos e POST único.
//! O 302 isolado não prova efeito; só releitura completa com id e texto.
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::conferir_endereco_relativo;
use crate::arvore_processo::endereco::EnderecoRecusado;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::consulta_do_controlador::menciona_acao;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    consulta_do_controlador, decodificar_atributo, derivar_submissao, enviar_submissao_via,
};
use crate::html_atributos::atributos;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_comentar::{FalhaDaConferencia, conferir_efeito};
use crate::processo_comentarios::{
    ErroComentarios, ErroLinkDosComentarios, TelaDosComentarios, ler_tela,
};
use crate::processo_concluir::escrita::{
    Contador, CookieIlegivel, FalhaDaTela, leitura, ler_cookie, obter_tela,
};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::visao_do_controle::EnvioDoControle as _;

const COMENTAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ComentarioCadastrar,
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

/// `--texto` vazio, só com espaços ou com NUL.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct TextoDoComentarioInvalido;

/// `--texto`: o clap recusa o inválido (saída 2) antes da sessão.
pub(crate) fn texto_do_comentario(texto: &str) -> Result<String, TextoDoComentarioInvalido> {
    if texto.trim().is_empty() || texto.contains('\0') {
        Err(TextoDoComentarioInvalido)
    } else {
        Ok(texto.to_owned())
    }
}

/// Um erro por passo, com a causa; nada foi gravado, salvo em `NaoConfirmado`.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroComentar {
    OrigemNaoAutorizada,
    Abertura(#[from] ErroProcessoObter),
    Barra(#[from] ErroLinkDosComentarios),
    LinkDaListaNaoPublicado,
    Cookie(#[from] CookieIlegivel),
    TelaDaLista(#[source] FalhaDaTela),
    ListaIlegivel(#[source] ErroComentarios),
    ListaForaDaForma,
    /// O tokenizador HTML recusou a lista de comentários.
    ListaHtmlNaoInterpretada(#[source] tl::ParseError),
    /// O tokenizador de atributos recusou o `href` de um link da lista.
    AtributoDaLinkIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
    /// Zero ou mais de um link `comentario_cadastrar` na lista.
    LinkDoCadastroNaoUnico,
    LinkDoCadastroRecusado(EnderecoRecusado),
    TelaDoCadastro(#[source] FalhaDaTela),
    CadastroIlegivel(#[source] ErroComentarios),
    CadastroForaDaForma,
    CharsetDoCadastroNaoSuportado,
    Formulario(#[source] ErroSubmissaoFormulario),
    EnderecoDoFormularioRecusado,
    EnderecoDoFormularioRecusadoPelaPolitica(EnderecoRecusado),
    /// O POST saiu e a releitura não comprovou o comentário: não repita
    /// `--confirmar` sem conferir a lista.
    NaoConfirmado {
        requisicoes: u8,
        #[source]
        causa: IncertezaDoComentario,
    },
}

/// Por que o POST enviado não foi comprovado.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum IncertezaDoComentario {
    Envio(#[source] ErroSubmissaoFormulario),
    RespostaSemRedirecionamento,
    AceiteSemIdentidade,
    Releitura(#[source] FalhaDaTela),
    ListaRelidaIlegivel(#[source] ErroComentarios),
    ListaRelidaForaDaForma,
    Conferencia(#[source] FalhaDaConferencia),
}

pub(crate) struct Previa {
    pub(crate) numero: String,
    pub(crate) id_procedimento: String,
    pub(crate) texto: String,
    pub(crate) requisicoes: u8,
    pub(crate) confirmado: bool,
}

/// Verdadeiro se a URL é a ação `comentario_cadastrar` (campo exato da consulta).
fn e_link_do_cadastro(url: &str) -> bool {
    menciona_acao(url, AcaoDoSei::ComentarioCadastrar)
}

/// O `onclick` que o SEI publica no botão Novo: exatamente `location.href='<url>'`.
fn url_do_botao_de_cadastro(onclick: &str) -> Option<&str> {
    let url = onclick
        .strip_prefix("location.href='")?
        .strip_suffix('\'')?;
    e_link_do_cadastro(url).then_some(url)
}

/// Encontra exatamente um link de Novo Comentário publicado no HTML da lista,
/// seja `<a href>` seja `<button onclick="location.href='…'">`.
/// A URL é conferida pela política canônica do controlador e pela identidade.
fn link_do_cadastro(instalacao: &Instalacao, html: &str, id: &str) -> Result<String, ErroComentar> {
    let fora = || ErroComentar::ListaForaDaForma;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroComentar::ListaHtmlNaoInterpretada)?;
    let mut links = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or_else(fora)?;
        let href = atributos(tag)
            .and_then(|a| a.valor("href"))
            .map_err(ErroComentar::AtributoDaLinkIlegivel)?;
        if let Some(href) = href {
            let href = decodificar_atributo(href).ok_or_else(fora)?;
            if e_link_do_cadastro(&href) {
                links.push(href);
            }
        }
    }
    for no in dom.query_selector("button").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or_else(fora)?;
        let onclick = atributos(tag)
            .and_then(|a| a.valor("onclick"))
            .map_err(ErroComentar::AtributoDaLinkIlegivel)?;
        if let Some(onclick) = onclick {
            let onclick = decodificar_atributo(onclick).ok_or_else(fora)?;
            if let Some(url) = url_do_botao_de_cadastro(&onclick) {
                links.push(url.to_owned());
            }
        }
    }
    let [unico] = links.as_slice() else {
        return Err(ErroComentar::LinkDoCadastroNaoUnico);
    };
    conferir_endereco_relativo(instalacao, unico, AcaoDoSei::ComentarioCadastrar, id)
        .map_err(ErroComentar::LinkDoCadastroRecusado)
}

/// Confere a identidade do aceite sem seguir o redirecionamento.
fn id_do_aceite<'a>(local: &'a str, url_post: &str, id_processo: &str) -> Option<&'a str> {
    let consulta = ConsultaDoControlador::ler(consulta_do_controlador(local, url_post)?).ok()?;
    // `id_procedimento` é opcional, mas repetido nunca casa.
    let processo_confere = match consulta
        .valores("id_procedimento")
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => true,
        [processo] => *processo == id_processo,
        _ => false,
    };
    let id = consulta.unico("id_comentario").ok()?;
    (consulta.acao() == Ok(AcaoDoSei::ComentarioListar) && processo_confere)
        .then_some(id)
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
) -> Result<(), ErroComentar> {
    let mut enviou = false;
    let retorno = enviar_submissao_via(submissao, sessao.cookie, |post, cookie| {
        enviou = true;
        rede.post(post, cookie)
    });
    let incerto = |rede: &Contador<'_, V>, causa| ErroComentar::NaoConfirmado {
        requisicoes: rede.feitas(),
        causa,
    };
    let retorno = match retorno {
        Ok(retorno) => retorno,
        Err(erro) if enviou => return Err(incerto(rede, IncertezaDoComentario::Envio(erro))),
        Err(erro) => return Err(ErroComentar::Formulario(erro)),
    };
    let RespostaDoEnvio::Redirecionamento(redir) = retorno else {
        return Err(incerto(
            rede,
            IncertezaDoComentario::RespostaSemRedirecionamento,
        ));
    };
    let Some(id_aceito) = redir
        .localizacao()
        .and_then(|local| id_do_aceite(local, redir.url_do_envio(), id_processo))
    else {
        return Err(incerto(rede, IncertezaDoComentario::AceiteSemIdentidade));
    };
    let html = match obter_tela(rede, cookie, link_lista) {
        Ok((html, _)) => html,
        Err(falha) => return Err(incerto(rede, IncertezaDoComentario::Releitura(falha))),
    };
    let relidos = match ler_tela(&html) {
        Ok(TelaDosComentarios::Lista(relidos)) => relidos,
        Ok(_) => return Err(incerto(rede, IncertezaDoComentario::ListaRelidaForaDaForma)),
        Err(erro) => {
            return Err(incerto(
                rede,
                IncertezaDoComentario::ListaRelidaIlegivel(erro),
            ));
        }
    };
    conferir_efeito(&relidos, id_aceito, texto)
        .map_err(|f| incerto(rede, IncertezaDoComentario::Conferencia(f)))
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
) -> Result<Previa, ErroComentar> {
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(ErroComentar::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(nup, unidade, false), indice, &mut rede)?;
    let link = acoes
        .comentarios?
        .ok_or(ErroComentar::LinkDaListaNaoPublicado)?;
    let cookie = ler_cookie(sessao)?;
    let (html, _) =
        obter_tela(&mut rede, &cookie, link.as_str()).map_err(ErroComentar::TelaDaLista)?;
    let TelaDosComentarios::Lista(_) = ler_tela(&html).map_err(ErroComentar::ListaIlegivel)? else {
        return Err(ErroComentar::ListaForaDaForma);
    };
    let id = &aberto.arvore.processo.id;
    let url = link_do_cadastro(&instalacao, &html, id)?;
    let (cadastro, mime) =
        obter_tela(&mut rede, &cookie, &url).map_err(ErroComentar::TelaDoCadastro)?;
    let TelaDosComentarios::CadastroSemComentarios =
        ler_tela(&cadastro).map_err(ErroComentar::CadastroIlegivel)?
    else {
        return Err(ErroComentar::CadastroForaDaForma);
    };
    let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
        .ok_or(ErroComentar::CharsetDoCadastroNaoSuportado)?;
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
        aberto.unidade.as_deref(),
        &[("txaDescricao", texto)],
    )
    .map_err(ErroComentar::Formulario)?;
    let endereco = submissao
        .url()
        .strip_prefix(&format!("{}/", instalacao.url_base()))
        .ok_or(ErroComentar::EnderecoDoFormularioRecusado)?;
    conferir_endereco_relativo(&instalacao, endereco, AcaoDoSei::ComentarioCadastrar, id)
        .map_err(ErroComentar::EnderecoDoFormularioRecusadoPelaPolitica)?;
    if confirmar_escrita {
        confirmar(
            submissao,
            sessao,
            &mut rede,
            &cookie,
            link.as_str(),
            id,
            texto,
        )?;
    }
    Ok(Previa {
        numero: nup.as_str().to_owned(),
        id_procedimento: id.to_owned(),
        texto: texto.to_owned(),
        requisicoes: rede.feitas(),
        confirmado: confirmar_escrita,
    })
}

#[cfg(test)]
mod acervo;
#[cfg(test)]
mod tests;
