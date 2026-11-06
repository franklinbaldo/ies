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
use crate::processo_concluir::escrita::{Contador, leitura, ler_cookie};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::{FalhaTransporte, decodificar_resposta};
use crate::visao_do_controle::EnvioDoControle as _;
#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum ErroPrevia {
    #[causa(interna)]
    Abertura(ErroProcessoObter),
    #[causa(nome = "origem_do_acompanhamento_nao_autorizada")]
    OrigemNaoAutorizada,
    #[causa(nome = "unidade_do_acompanhamento_nao_publicada")]
    UnidadeNaoPublicada,
    #[causa(nome = "acompanhamento_nao_publicado_na_barra")]
    LinkNaoPublicado,
    #[causa(interna)]
    Barra(ErroAcompanharNaBarra),
    #[causa(interna)]
    Transporte(FalhaTransporte),
    #[causa(interna)]
    Estrutura(ErroAcompanhar),
    #[causa(interna)]
    Formulario(crate::formulario_web::ErroSubmissaoFormulario),
    #[causa(nome = "acompanhamento_nao_confirmado_por_leitura")]
    EfeitoIncerto,
    #[causa(nome = "confirmacao_de_acompanhamento_ainda_bloqueada")]
    ConfirmacaoNaoImplementada,
    #[causa(transparente)]
    Html(ErroHtml),
}

impl ErroPrevia {
    pub(crate) fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }
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
    pub(crate) ja_estava: bool,
    pub(crate) confirmado: bool,
    pub(crate) alterado: bool,
    pub(crate) requisicoes: u8,
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
            .trim()
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
        let alvo = if antes.is_empty() {
            None
        } else {
            Some(alvo_para_deixar(antes, pedido.grupo)?)
        };
        return Ok(Previa {
            numero: numero.to_owned(),
            unidade: unidade.to_owned(),
            operacao: "deixar",
            grupo: alvo.map_or("", |a| a.grupo.as_str()).to_owned(),
            observacao: alvo.map_or("", |a| a.observacao.as_str()).to_owned(),
            id_de_remocao: alvo.map(|a| a.id.clone()),
            ja_estava: alvo.is_none(),
            confirmado: false,
            alterado: false,
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
        ja_estava: super::conferir_novo_grupo(antes, &selecionado.nome).is_err(),
        confirmado: false,
        alterado: false,
        requisicoes,
    })
}

type LeituraGerenciador = (
    Vec<Acompanhamento>,
    Option<String>,
    Option<SubmissaoDerivada>,
);

/// Lê o gerenciador e a oferta de exclusão, sem enviar escrita.
fn ler_gerenciador<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    instalacao: &Instalacao,
    id: &str,
    unidade: &str,
    link: &str,
    cookie: &str,
    pedido: &Pedido<'_>,
) -> Result<LeituraGerenciador, ErroPrevia> {
    let res = rede.get_sem_seguir(link, cookie)?;
    let mut submissao: Option<SubmissaoDerivada> = None;
    let (antes, url_cadastro) = if res.resposta.status_code == 302 {
        let url = redirecionamento_do_gerenciar(instalacao, res.localizacao.as_deref(), link, id)?;
        // O redirect, sozinho, não prova uma lista vazia. A tela de
        // cadastro deve ser lida e identificada no processo correto.
        if pedido.deixar {
            let (html, _) = html_da_tela(rede, cookie, &url)?;
            grupos_do_cadastro(&html, id)?;
            (Vec::new(), None)
        } else {
            (Vec::new(), Some(url))
        }
    } else {
        let mime = res.resposta.mime.clone();
        let html = decodificar_resposta(res.resposta)?;
        let antes = acompanhamentos_da_lista(&html)?;
        if pedido.deixar && !antes.is_empty() {
            let alvo = alvo_para_deixar(&antes, pedido.grupo)?;
            let destino =
                super::exclusao::conferir_oferta_exclusao(instalacao, &html, id, &alvo.id)?;
            if pedido.confirmar {
                let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
                    .ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
                let tela = crate::formulario_web::TelaServida {
                    url: link,
                    html: &html,
                    codificacao,
                };
                submissao = Some(super::submissao::derivar_exclusao(
                    instalacao, &tela, unidade, &alvo.id, &destino,
                )?);
            }
        }
        let url = if pedido.deixar {
            None
        } else {
            Some(adicionar_da_lista(instalacao, &html, id)?)
        };
        (antes, url)
    };
    Ok((antes, url_cadastro, submissao))
}

/// Prévia completa sem escrita, no máximo um GET da lista, mais um do cadastro.
pub(crate) fn previa<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, ErroPrevia> {
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
    let (antes, url_cadastro, mut submissao) = ler_gerenciador(
        &mut rede,
        &instalacao,
        &id,
        &unidade,
        link.as_str(),
        &cookie,
        pedido,
    )?;
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
    plano.confirmado = true;
    if plano.ja_estava {
        return Ok(plano);
    }
    let submissao = submissao.ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?;
    if pedido.deixar {
        return confirmar_remocao(
            &mut rede,
            sessao,
            &ContextoRemocao {
                instalacao: &instalacao,
                id: &id,
                link: link.as_str(),
                cookie: &cookie,
                antes: &antes,
            },
            plano,
            submissao,
        );
    }
    confirmar_inclusao(
        &mut rede,
        sessao,
        &instalacao,
        &id,
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
    instalacao: &Instalacao,
    id: &str,
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
    // A releitura prova o efeito, mas não legitima um 302 para outra
    // operação ou outro processo. Sem resposta de rede, o POST ainda
    // pode ter sido consumido: apenas uma leitura exata pode conciliá-lo.
    let aceite = resposta.is_err() || aceite_do_gerenciador(&resposta, instalacao, id);
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
    if !aceite || !verificado {
        return Err(ErroPrevia::EfeitoIncerto);
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
        .ok_or(ErroPrevia::EfeitoIncerto)?;
    plano.id_de_remocao = Some(novo.id.clone());
    plano.alterado = true;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}

/// O 302 do POST precisa apontar ao gerenciador do mesmo processo.
/// Usado na inclusão e retirada; nenhuma resposta é seguida automaticamente.
fn aceite_do_gerenciador(
    resposta: &Result<RespostaDoEnvio, crate::formulario_web::ErroSubmissaoFormulario>,
    instalacao: &Instalacao,
    id: &str,
) -> bool {
    let Ok(RespostaDoEnvio::Redirecionamento(redirecionamento)) = resposta else {
        return false;
    };
    let Some(consulta) = redirecionamento.localizacao().and_then(|loc| {
        crate::formulario_web::consulta_do_controlador(loc, redirecionamento.url_do_envio())
    }) else {
        return false;
    };
    crate::arvore_processo::conferir_endereco_relativo(
        instalacao,
        &format!("controlador.php?{consulta}"),
        "acompanhamento_gerenciar",
        id,
    )
    .is_ok()
}

/// A última retirada pode tornar o gerenciador um 302 para o cadastro.
/// Só o cadastro válido do processo, lido por GET explícito, comprova
/// a ausência de linhas; HTML truncado ou redirect estranho são incertos.
fn releitura_da_remocao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    instalacao: &Instalacao,
    id: &str,
    link: &str,
    cookie: &str,
) -> Option<Vec<Acompanhamento>> {
    let res = rede.get_sem_seguir(link, cookie).ok()?;
    if res.resposta.status_code == 302 {
        let url =
            redirecionamento_do_gerenciar(instalacao, res.localizacao.as_deref(), link, id).ok()?;
        let (html, _) = html_da_tela(rede, cookie, &url).ok()?;
        grupos_do_cadastro(&html, id).ok()?;
        return Some(Vec::new());
    }
    let html = decodificar_resposta(res.resposta).ok()?;
    acompanhamentos_da_lista(&html).ok()
}

/// Elementos observados na mesma sessão e abertura que originaram o
/// formulário da remoção. O POST é consumido uma vez.
struct ContextoRemocao<'a> {
    instalacao: &'a Instalacao,
    id: &'a str,
    link: &'a str,
    cookie: &'a str,
    antes: &'a [Acompanhamento],
}

/// A escrita nunca se repete, nem após erro de rede. O único critério de
/// sucesso é o aceite observado E a leitura nova com desaparecimento
/// exclusivo da linha alvo, preservando todas as demais identidades.
fn confirmar_remocao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    contexto: &ContextoRemocao<'_>,
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
    let aceite = aceite_do_gerenciador(&resposta, contexto.instalacao, contexto.id);
    let depois = releitura_da_remocao(
        rede,
        contexto.instalacao,
        contexto.id,
        contexto.link,
        contexto.cookie,
    );
    let verificado = depois.as_deref().is_some_and(|linhas| {
        super::conferir_remocao(
            contexto.antes,
            linhas,
            plano.id_de_remocao.as_deref().unwrap_or_default(),
        )
        .is_ok()
    });
    if !aceite || !verificado {
        return Err(ErroPrevia::EfeitoIncerto);
    }
    plano.alterado = true;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_remocao.rs"]
mod tests_remocao;
