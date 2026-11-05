//! Prévia Web para acompanhamento especial (#254).
//! GET da barra, 302 tratado como dado, GET só do cadastro quando necessário;
//! nunca dispara o POST antes da revisão do fluxo de confirmação.
use super::{
    Acompanhamento, ErroAcompanhar, acompanhamentos_da_lista, alvo_para_deixar,
    conferir_inclusao, escolher_grupo, grupos_do_cadastro,
};
use crate::arvore_processo::ErroAcompanharNaBarra;
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
#[derive(Debug)]
pub(crate) enum ErroPrevia {
    Abertura(ErroProcessoObter),
    OrigemNaoAutorizada,
    UnidadeNaoPublicada,
    LinkNaoPublicado,
    Barra(ErroAcompanharNaBarra),
    Transporte(FalhaTransporte),
    RedirecionamentoInesperado,
    TelaInesperada,
    Estrutura(ErroAcompanhar),
    Formulario(crate::formulario_web::ErroSubmissaoFormulario),
    ConfirmacaoNaoImplementada,
    EfeitoIncerto,
    PedidoInvalido,
}

impl ErroPrevia {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.subcodigo(),
            Self::OrigemNaoAutorizada => "origem_do_acompanhamento_nao_autorizada",
            Self::UnidadeNaoPublicada => "unidade_do_acompanhamento_nao_publicada",
            Self::LinkNaoPublicado => "acompanhamento_nao_publicado_na_barra",
            Self::Barra(erro) => erro.subcodigo(),
            Self::Transporte(erro) => erro.subcodigo(),
            Self::RedirecionamentoInesperado => "redirecionamento_de_acompanhamento_inesperado",
            Self::TelaInesperada => "tela_de_acompanhamento_inesperada",
            Self::Estrutura(erro) => erro.subcodigo(),
            Self::Formulario(erro) => erro.subcodigo(),
            Self::ConfirmacaoNaoImplementada => "confirmacao_de_acompanhamento_ainda_bloqueada",
            Self::EfeitoIncerto => "acompanhamento_nao_confirmado_por_leitura",
            Self::PedidoInvalido => "pedido_de_acompanhamento_invalido",
        }
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
    conferir_endereco_relativo(instalacao, ref_url, "acompanhamento_cadastrar", id)
        .map_err(|_| ErroPrevia::RedirecionamentoInesperado)
}

fn redirecionamento_do_gerenciar(
    instalacao: &Instalacao,
    loc: Option<&str>,
    origem: &str,
    id: &str,
) -> Result<String, ErroPrevia> {
    let consulta = loc
        .and_then(|v| crate::formulario_web::consulta_do_controlador(v, origem))
        .ok_or(ErroPrevia::RedirecionamentoInesperado)?;
    url_do_cadastro(instalacao, &format!("controlador.php?{consulta}"), id)
}

/// Apenas `btnAdicionar` da lista pode abrir um cadastro adicional.
fn adicionar_da_lista(instalacao: &Instalacao, html: &str, id: &str) -> Result<String, ErroPrevia> {
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroPrevia::TelaInesperada)?;
    let mut urls = Vec::new();
    for no in dom.query_selector("button").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(ErroPrevia::TelaInesperada)?;
        let identificador = crate::html_atributos::valor_do_atributo(tag, "id")
            .map_err(|_| ErroPrevia::TelaInesperada)?;
        if identificador.as_deref() != Some("btnAdicionar") {
            continue;
        }
        let onclick = crate::html_atributos::valor_do_atributo(tag, "onclick")
            .map_err(|_| ErroPrevia::TelaInesperada)?
            .ok_or(ErroPrevia::TelaInesperada)?;
        let onclick = decodificar_atributo(&onclick).ok_or(ErroPrevia::TelaInesperada)?;
        let url = onclick
            .trim()
            .strip_prefix("location.href='")
            .and_then(|v| v.strip_suffix("';").or_else(|| v.strip_suffix("'")))
            .ok_or(ErroPrevia::TelaInesperada)?;
        urls.push(url_do_cadastro(instalacao, url, id)?);
    }
    match urls.as_slice() {
        [url] => Ok(url.to_owned()),
        _ => Err(ErroPrevia::TelaInesperada),
    }
}

/// O HTML deve ser 200 sem tela de login e sem redirect automático.
fn html_da_tela<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
) -> Result<(String, String), ErroPrevia> {
    let resposta = rede
        .get_sem_seguir(url, cookie)
        .map_err(ErroPrevia::Transporte)?;
    if resposta.resposta.status_code == 302 {
        return Err(ErroPrevia::RedirecionamentoInesperado);
    }
    let mime = resposta.resposta.mime.clone();
    let html = decodificar_resposta(resposta.resposta).map_err(ErroPrevia::Transporte)?;
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
            return Err(ErroPrevia::PedidoInvalido);
        }
        let alvo = if antes.is_empty() {
            None
        } else {
            Some(alvo_para_deixar(antes, pedido.grupo).map_err(ErroPrevia::Estrutura)?)
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
    let grupos = grupos.ok_or(ErroPrevia::TelaInesperada)?;
    let selecionado = escolher_grupo(grupos, pedido.grupo).map_err(ErroPrevia::Estrutura)?;
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
        return Err(ErroPrevia::PedidoInvalido);
    }
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(ErroPrevia::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )
    .map_err(ErroPrevia::Abertura)?;
    let unidade = aberto.unidade.ok_or(ErroPrevia::UnidadeNaoPublicada)?;
    let id = aberto.arvore.processo.id;
    let link = acoes
        .acompanhar
        .map_err(ErroPrevia::Barra)?
        .ok_or(ErroPrevia::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao).map_err(ErroPrevia::Transporte)?;
    let res = rede
        .get_sem_seguir(link.as_str(), &cookie)
        .map_err(ErroPrevia::Transporte)?;
    let (antes, url_cadastro) = if res.resposta.status_code == 302 {
        let url = redirecionamento_do_gerenciar(
            &instalacao,
            res.localizacao.as_deref(),
            link.as_str(),
            &id,
        )?;
        (Vec::new(), Some(url))
    } else {
        let html = decodificar_resposta(res.resposta).map_err(ErroPrevia::Transporte)?;
        let antes = acompanhamentos_da_lista(&html).map_err(ErroPrevia::Estrutura)?;
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
        let url = url_cadastro.ok_or(ErroPrevia::TelaInesperada)?;
        let (html, mime) = html_da_tela(&mut rede, &cookie, &url)?;
        let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
            .ok_or(ErroPrevia::TelaInesperada)?;
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
        Some(grupos_do_cadastro(&html, &id).map_err(ErroPrevia::Estrutura)?)
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
    let submissao = submissao.ok_or(ErroPrevia::TelaInesperada)?;
    // Enviar uma única vez. Falha no POST pode ter produzido efeito;
    // neste caso, ainda é necessário consultar a lista após o envio.
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |formulario, cookie| {
        enviado = true;
        rede.post(formulario, cookie)
    });
    if !enviado {
        return Err(ErroPrevia::Formulario(
            resposta.err().ok_or(ErroPrevia::TelaInesperada)?,
        ));
    }
    // O 302 é dado, nunca seguido. A resposta não comprova o efeito.
    let _aceite_302 = matches!(resposta, Ok(RespostaDoEnvio::Redirecionamento(_)));
    let depois = rede.get_sem_seguir(link.as_str(), &cookie);
    let depois = match depois {
        Ok(resposta) if resposta.resposta.status_code != 302 => {
            decodificar_resposta(resposta.resposta)
                .ok()
                .and_then(|html| acompanhamentos_da_lista(&html).ok())
        }
        _ => None,
    };
    let verificado = depois.as_deref().is_some_and(|linhas| {
        conferir_inclusao(&antes, linhas, &plano.grupo, &plano.observacao).is_ok()
    });
    if !verificado {
        return Err(ErroPrevia::EfeitoIncerto);
    }
    plano.alterado = true;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}
