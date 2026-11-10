//! Prévia e inclusão confirmada de Acompanhamento Especial (#254).
//! GET da barra e cadastro, POST único somente com --confirmar,
//! seguido por GET explícito de verificação; nenhum redirect seguido.
use super::{
    Acompanhamento, ErroAcompanhar, acompanhamentos_da_lista, alvo_para_deixar, conferir_inclusao,
    escolher_grupo, grupos_do_cadastro,
};
use crate::arvore_processo::ErroAcompanharNaBarra;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    RespostaDoEnvio, SubmissaoDerivada, decodificar_atributo, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{Contador, leitura, ler_cookie};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::{FalhaTransporte, decodificar_resposta};
use crate::visao_do_controle::EnvioDoControle as _;
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroPrevia {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(#[from] CookieIlegivel),
    Abertura(#[from] ErroProcessoObter),
    OrigemNaoAutorizada,
    UnidadeNaoPublicada,
    LinkNaoPublicado,
    Barra(#[from] ErroAcompanharNaBarra),
    Transporte(#[from] FalhaTransporte),
    Estrutura(#[from] ErroAcompanhar),
    Formulario(#[from] crate::formulario_web::ErroSubmissaoFormulario),
    ConfirmacaoNaoImplementada,
    NaoConfirmado,
    Html(#[from] ErroHtml),
}

pub(crate) struct Pedido<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) grupo: Option<&'a str>,
    pub(crate) observacao: Option<&'a str>,
    pub(crate) deixar: bool,
    pub(crate) confirmar: bool,
}

#[derive(Debug)]
pub(crate) struct Previa {
    pub(crate) numero: String,
    pub(crate) unidade: String,
    pub(crate) operacao: &'static str,
    pub(crate) grupo: String,
    pub(crate) observacao: String,
    pub(crate) id_de_remocao: Option<String>,
    pub(crate) estado: EstadoDoAcompanhamento,
    pub(crate) requisicoes: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDoAcompanhamento {
    Previa { ja_estava: bool },
    ConfirmadoSemMudanca,
    Incluido,
}

impl EstadoDoAcompanhamento {
    pub(crate) fn ja_estava(self) -> bool {
        matches!(
            self,
            Self::Previa { ja_estava: true } | Self::ConfirmadoSemMudanca
        )
    }

    pub(crate) fn confirmado(self) -> bool {
        matches!(self, Self::ConfirmadoSemMudanca | Self::Incluido)
    }

    pub(crate) fn alterado(self) -> bool {
        matches!(self, Self::Incluido)
    }
}

fn url_do_cadastro(instalacao: &Instalacao, ref_url: &str, id: &str) -> Result<String, ErroPrevia> {
    use crate::arvore_processo::conferir_endereco_relativo;
    conferir_endereco_relativo(instalacao, ref_url, "acompanhamento_cadastrar", id).map_err(|_| {
        ErroPrevia::Html(ErroHtml::valor(
            "redirecionamento_de_acompanhamento_inesperado",
        ))
    })
}

fn redirecionamento_do_gerenciar(
    instalacao: &Instalacao,
    loc: Option<&str>,
    origem: &str,
    id: &str,
) -> Result<String, ErroPrevia> {
    let consulta = loc
        .and_then(|v| crate::formulario_web::consulta_do_controlador(v, origem))
        .ok_or(ErroHtml::valor(
            "redirecionamento_de_acompanhamento_inesperado",
        ))?;
    url_do_cadastro(instalacao, &format!("controlador.php?{consulta}"), id)
}

/// Apenas `btnAdicionar` da lista pode abrir um cadastro adicional.
fn adicionar_da_lista(instalacao: &Instalacao, html: &str, id: &str) -> Result<String, ErroPrevia> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
    let mut urls = Vec::new();
    for no in dom.query_selector("button").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
        let identificador = crate::html_atributos::valor_do_atributo(tag, "id")
            .map_err(|e| ErroHtml::atributo("tela_de_acompanhamento_inesperada", e))?;
        if identificador.as_deref() != Some("btnAdicionar") {
            continue;
        }
        let onclick = crate::html_atributos::valor_do_atributo(tag, "onclick")
            .map_err(|e| ErroHtml::atributo("tela_de_acompanhamento_inesperada", e))?
            .ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
        let onclick = decodificar_atributo(&onclick)
            .ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
        let url = onclick
            .strip_prefix("location.href='")
            .and_then(|v| v.strip_suffix("';").or_else(|| v.strip_suffix("'")))
            .ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
        urls.push(url_do_cadastro(instalacao, url, id)?);
    }
    match urls.as_slice() {
        [url] => Ok(url.to_owned()),
        _ => Err(ErroHtml::valor("tela_de_acompanhamento_inesperada").into()),
    }
}

/// O HTML deve ser 200 sem tela de login e sem redirect automático.
fn html_da_tela<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
) -> Result<(String, String), ErroPrevia> {
    let resposta = rede.get_sem_seguir(url, cookie)?;
    if resposta.resposta.status_code == 302 {
        return Err(ErroHtml::valor("redirecionamento_de_acompanhamento_inesperado").into());
    }
    let mime = resposta.resposta.mime.clone();
    let html = decodificar_resposta(resposta.resposta)?;
    Ok((html, mime))
}

fn selecionar_previa(
    numero: &str,
    unidade: &str,
    antes: &[Acompanhamento],
    grupos: Option<&[super::GrupoOferecido]>,
    pedido: &Pedido<'_>,
    requisicoes: u8,
) -> Result<Previa, ErroPrevia> {
    if pedido.deixar {
        if pedido.observacao.is_some() {
            return Err(ErroHtml::valor("pedido_de_acompanhamento_invalido").into());
        }
        let alvo = if antes.is_empty()
            || pedido
                .grupo
                .is_some_and(|grupo| !antes.iter().any(|a| a.grupo == grupo))
        {
            None
        } else {
            Some(alvo_para_deixar(antes, pedido.grupo)?)
        };
        return Ok(Previa {
            numero: numero.to_owned(),
            unidade: unidade.to_owned(),
            operacao: "deixar",
            grupo: alvo
                .map_or(pedido.grupo.unwrap_or(""), |a| a.grupo.as_str())
                .to_owned(),
            observacao: alvo.map_or("", |a| a.observacao.as_str()).to_owned(),
            id_de_remocao: alvo.map(|a| a.id.clone()),
            estado: EstadoDoAcompanhamento::Previa {
                ja_estava: alvo.is_none(),
            },
            requisicoes,
        });
    }
    let grupos = grupos.ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
    let selecionado = escolher_grupo(grupos, pedido.grupo)?;
    Ok(Previa {
        numero: numero.to_owned(),
        unidade: unidade.to_owned(),
        operacao: "acompanhar",
        grupo: selecionado.nome.clone(),
        observacao: pedido.observacao.unwrap_or("").to_owned(),
        id_de_remocao: None,
        estado: EstadoDoAcompanhamento::Previa {
            ja_estava: super::conferir_novo_grupo(antes, &selecionado.nome).is_err(),
        },
        requisicoes,
    })
}

/// Prévia completa sem escrita, no máximo um GET da lista, mais um do cadastro.
pub(crate) fn previa<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, ErroPrevia> {
    // A exclusão exige formulário próprio; só a inclusão pode confirmar.
    if pedido.confirmar && pedido.deixar {
        return Err(ErroPrevia::ConfirmacaoNaoImplementada);
    }
    if pedido.deixar && pedido.observacao.is_some() {
        return Err(ErroHtml::valor("pedido_de_acompanhamento_invalido").into());
    }
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(ErroPrevia::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )?;
    let unidade = aberto.unidade.ok_or(ErroPrevia::UnidadeNaoPublicada)?;
    let id = aberto.arvore.processo.id;
    let link = acoes.acompanhar?.ok_or(ErroPrevia::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao)?;
    let res = rede.get_sem_seguir(link.as_str(), &cookie)?;
    let (antes, url_cadastro) = if res.resposta.status_code == 302 {
        let url = redirecionamento_do_gerenciar(
            &instalacao,
            res.localizacao.as_deref(),
            link.as_str(),
            &id,
        )?;
        (Vec::new(), Some(url))
    } else {
        let html = decodificar_resposta(res.resposta)?;
        let antes = acompanhamentos_da_lista(&html)?;
        let url = if pedido.deixar {
            None
        } else {
            Some(adicionar_da_lista(&instalacao, &html, &id)?)
        };
        (antes, url)
    };
    let mut submissao: Option<SubmissaoDerivada> = None;
    let grupos = if pedido.deixar {
        None
    } else {
        let url = url_cadastro.ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
        let (html, mime) = html_da_tela(&mut rede, &cookie, &url)?;
        let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
            .ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
        // A prévia só é considerada válida se a submissão exata também
        // puder ser derivada, sem escrever e sem construir URL artificial.
        let tela = crate::formulario_web::TelaServida {
            url: &url,
            html: &html,
            codificacao,
        };
        let derivada = super::submissao::derivar_inclusao(
            &instalacao,
            &tela,
            &id,
            &unidade,
            pedido.grupo,
            pedido.observacao.unwrap_or(""),
        )?;
        if pedido.confirmar {
            submissao = Some(derivada);
        }
        Some(grupos_do_cadastro(&html, &id)?)
    };
    let mut plano = selecionar_previa(
        pedido.nup.as_str(),
        &unidade,
        &antes,
        grupos.as_deref(),
        pedido,
        rede.feitas(),
    )?;
    if !pedido.confirmar {
        return Ok(plano);
    }
    if plano.estado.ja_estava() {
        plano.estado = EstadoDoAcompanhamento::ConfirmadoSemMudanca;
        return Ok(plano);
    }
    let submissao = submissao.ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
    confirmar_inclusao(
        &mut rede,
        sessao,
        link.as_str(),
        &cookie,
        &antes,
        plano,
        submissao,
    )
}

/// Consome uma única submissão e verifica o resultado por leitura nova.
/// Mesmo um erro de transporte no POST pode ter produzido efeito.
fn confirmar_inclusao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    link: &str,
    cookie: &str,
    antes: &[Acompanhamento],
    mut plano: Previa,
    submissao: SubmissaoDerivada,
) -> Result<Previa, ErroPrevia> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |formulario, cookie| {
        enviado = true;
        rede.post(formulario, cookie)
    });
    if !enviado {
        return Err(ErroPrevia::Formulario(
            resposta
                .err()
                .ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?,
        ));
    }
    // O 302 é dado, nunca seguido; só a lista relida pode provar efeito.
    let _aceite_302 = matches!(resposta, Ok(RespostaDoEnvio::Redirecionamento(_)));
    let depois = rede.get_sem_seguir(link, cookie);
    let depois = match depois {
        Ok(resposta) if resposta.resposta.status_code != 302 => {
            decodificar_resposta(resposta.resposta)
                .ok()
                .and_then(|html| acompanhamentos_da_lista(&html).ok())
        }
        _ => None,
    };
    let verificado = depois.as_deref().is_some_and(|linhas| {
        conferir_inclusao(antes, linhas, &plano.grupo, &plano.observacao).is_ok()
    });
    if !verificado {
        return Err(ErroPrevia::NaoConfirmado);
    }
    // Somente depois da prova exata podemos publicar o ID observado.
    let novo = depois
        .as_deref()
        .and_then(|linhas| {
            linhas.iter().find(|linha| {
                !antes.iter().any(|anterior| anterior.id == linha.id)
                    && linha.grupo == plano.grupo
                    && linha.observacao == plano.observacao
            })
        })
        .ok_or(ErroPrevia::NaoConfirmado)?;
    plano.id_de_remocao = Some(novo.id.clone());
    plano.estado = EstadoDoAcompanhamento::Incluido;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
mod tests;
