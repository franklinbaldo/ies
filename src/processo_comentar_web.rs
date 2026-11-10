//! Prévia e confirmação de comentário por links servidos e POST único.
//! O 302 isolado não prova efeito; só releitura completa com id e texto.
use crate::arvore_processo::conferir_endereco_relativo;
use crate::arvore_processo::endereco::EnderecoRecusado;
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

/// Um passo por variante, com a causa; nada foi gravado, salvo em `NaoConfirmado`.
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
    /// Zero ou mais de um link `comentario_cadastrar` na lista.
    LinkDoCadastroNaoUnico,
    LinkDoCadastroRecusado(EnderecoRecusado),
    TelaDoCadastro(#[source] FalhaDaTela),
    CadastroIlegivel(#[source] ErroComentarios),
    CadastroForaDaForma,
    CharsetDoCadastroNaoSuportado,
    Formulario(#[source] ErroSubmissaoFormulario),
    EnderecoDoFormularioRecusado,
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

/// Encontra exatamente um link de Novo Comentário publicado no HTML da lista.
/// A URL é conferida pela política canônica do controlador e pela identidade.
fn link_do_cadastro(instalacao: &Instalacao, html: &str, id: &str) -> Result<String, ErroComentar> {
    let fora = || ErroComentar::ListaForaDaForma;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| fora())?;
    let mut links = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or_else(fora)?;
        let href = atributos(tag)
            .and_then(|a| a.valor("href"))
            .map_err(|_| fora())?;
        if let Some(href) = href {
            let href = decodificar_atributo(href).ok_or_else(fora)?;
            if href
                .split(['?', '&'])
                .any(|campo| campo == "acao=comentario_cadastrar")
            {
                links.push(href);
            }
        }
    }
    let [unico] = links.as_slice() else {
        return Err(ErroComentar::LinkDoCadastroNaoUnico);
    };
    conferir_endereco_relativo(instalacao, unico, "comentario_cadastrar", id)
        .map_err(ErroComentar::LinkDoCadastroRecusado)
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
) -> Result<(), ErroComentar> {
    use IncertezaDoComentario as I;
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
        Err(erro) if enviou => return Err(incerto(rede, I::Envio(erro))),
        Err(erro) => return Err(ErroComentar::Formulario(erro)),
    };
    let RespostaDoEnvio::Redirecionamento(redir) = retorno else {
        return Err(incerto(rede, I::RespostaSemRedirecionamento));
    };
    let Some(id_aceito) = redir
        .localizacao()
        .and_then(|local| id_do_aceite(local, redir.url_do_envio(), id_processo))
    else {
        return Err(incerto(rede, I::AceiteSemIdentidade));
    };
    let html = match obter_tela(rede, cookie, link_lista) {
        Ok((html, _)) => html,
        Err(falha) => return Err(incerto(rede, I::Releitura(falha))),
    };
    let relidos = match ler_tela(&html) {
        Ok(TelaDosComentarios::Lista(relidos)) => relidos,
        Ok(_) => return Err(incerto(rede, I::ListaRelidaForaDaForma)),
        Err(erro) => return Err(incerto(rede, I::ListaRelidaIlegivel(erro))),
    };
    conferir_efeito(&relidos, id_aceito, texto).map_err(|f| incerto(rede, I::Conferencia(f)))
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
    use ErroComentar as E;
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(E::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(nup, unidade, false), indice, &mut rede)?;
    let link = acoes.comentarios?.ok_or(E::LinkDaListaNaoPublicado)?;
    let cookie = ler_cookie(sessao)?;
    let (html, _) = obter_tela(&mut rede, &cookie, link.as_str()).map_err(E::TelaDaLista)?;
    let TelaDosComentarios::Lista(_) = ler_tela(&html).map_err(E::ListaIlegivel)? else {
        return Err(E::ListaForaDaForma);
    };
    let id = &aberto.arvore.processo.id;
    let url = link_do_cadastro(&instalacao, &html, id)?;
    let (cadastro, mime) = obter_tela(&mut rede, &cookie, &url).map_err(E::TelaDoCadastro)?;
    let TelaDosComentarios::CadastroSemComentarios =
        ler_tela(&cadastro).map_err(E::CadastroIlegivel)?
    else {
        return Err(E::CadastroForaDaForma);
    };
    let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
        .ok_or(E::CharsetDoCadastroNaoSuportado)?;
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
    .map_err(E::Formulario)?;
    let endereco = submissao
        .url()
        .strip_prefix(&format!("{}/", instalacao.url_base()))
        .ok_or(E::EnderecoDoFormularioRecusado)?;
    conferir_endereco_relativo(&instalacao, endereco, "comentario_cadastrar", id)
        .map_err(|_| E::EnderecoDoFormularioRecusado)?;
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
