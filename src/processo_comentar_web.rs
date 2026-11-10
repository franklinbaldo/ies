//! Prévia de comentário: navega apenas por links servidos, sem POST.
//! A interpretação de 302 sem comentários e a confirmação ficam para outra fatia.
use crate::arvore_processo::conferir_endereco_relativo;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, TelaServida, decodificar_atributo, derivar_submissao,
};
use crate::html_atributos::atributos;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_comentarios::{TelaDosComentarios, ler_tela};
use crate::processo_concluir::escrita::{Contador, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso, abrir_para_escrever};

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
}

pub(crate) struct Previa {
    pub(crate) numero: String,
    pub(crate) id_procedimento: String,
    pub(crate) texto: String,
    pub(crate) requisicoes: u8,
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

/// Abre o formulário publicado pela lista do processo e não envia escrita.
///
/// # Errors
/// Recusa barra, lista, link ou formulário não comprovado.
pub(crate) fn prever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    texto: &str,
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
    let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
        .ok_or(Erro::Charset)?;
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
    Ok(Previa {
        numero: nup.as_str().to_owned(),
        id_procedimento: id.to_owned(),
        texto: texto.to_owned(),
        requisicoes: rede.feitas(),
    })
}
