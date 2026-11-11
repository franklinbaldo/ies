//! Prévia e inclusão confirmada de Acompanhamento Especial (#254).
//! GET da barra e cadastro, POST único somente com --confirmar,
//! seguido por GET explícito de verificação; nenhum redirect seguido.
use super::{
    Acompanhamento, ErroAcompanhar, acompanhamentos_da_lista, alvo_para_deixar, conferir_inclusao,
    escolher_grupo, grupos_do_cadastro,
};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    RespostaDoEnvio, SubmissaoDerivada, decodificar_atributo, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, leitura, ler_cookie};
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso, abrir_para_escrever};
use crate::transporte::decodificar_resposta;
use crate::visao_do_controle::EnvioDoControle as _;

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
    Removido,
}

impl EstadoDoAcompanhamento {
    pub(crate) fn ja_estava(self) -> bool {
        matches!(
            self,
            Self::Previa { ja_estava: true } | Self::ConfirmadoSemMudanca
        )
    }

    pub(crate) fn confirmado(self) -> bool {
        matches!(
            self,
            Self::ConfirmadoSemMudanca | Self::Incluido | Self::Removido
        )
    }

    pub(crate) fn alterado(self) -> bool {
        matches!(self, Self::Incluido | Self::Removido)
    }
}

fn url_do_cadastro(
    instalacao: &Instalacao,
    ref_url: &str,
    id: &str,
) -> Result<String, ErroAcompanhar> {
    use crate::arvore_processo::conferir_endereco_relativo;
    conferir_endereco_relativo(instalacao, ref_url, AcaoDoSei::AcompanhamentoCadastrar, id)
        .map_err(ErroAcompanhar::EnderecoDoCadastroRecusado)
}

fn redirecionamento_do_gerenciar(
    instalacao: &Instalacao,
    loc: Option<&str>,
    origem: &str,
    id: &str,
) -> Result<String, ErroAcompanhar> {
    let consulta = loc
        .and_then(|v| crate::formulario_web::consulta_do_controlador(v, origem))
        .ok_or(ErroAcompanhar::RedirecionamentoDeAcompanhamentoInesperado)?;
    url_do_cadastro(instalacao, &format!("controlador.php?{consulta}"), id)
}

/// Apenas `btnAdicionar` da lista pode abrir um cadastro adicional.
fn adicionar_da_lista(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<String, ErroAcompanhar> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroAcompanhar::TelaDeAcompanhamentoRecusadaPeloParser)?;
    let mut urls = Vec::new();
    for no in dom.query_selector("button").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
        let identificador = crate::html_atributos::valor_do_atributo(tag, "id")
            .map_err(ErroAcompanhar::Atributo)?;
        if identificador.as_deref() != Some("btnAdicionar") {
            continue;
        }
        let onclick = crate::html_atributos::valor_do_atributo(tag, "onclick")
            .map_err(ErroAcompanhar::Atributo)?
            .ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
        let onclick =
            decodificar_atributo(&onclick).ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
        let url = onclick
            .strip_prefix("location.href='")
            .and_then(|v| v.strip_suffix("';").or_else(|| v.strip_suffix("'")))
            .ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
        urls.push(url_do_cadastro(instalacao, url, id)?);
    }
    match urls.as_slice() {
        [url] => Ok(url.to_owned()),
        _ => Err(ErroAcompanhar::TelaDeAcompanhamentoInesperada),
    }
}

/// O HTML deve ser 200 sem tela de login e sem redirect automático.
fn html_da_tela<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
) -> Result<(String, String), ErroAcompanhar> {
    let resposta = rede.get_sem_seguir(url, cookie)?;
    if resposta.resposta.status_code == 302 {
        return Err(ErroAcompanhar::RedirecionamentoDeAcompanhamentoInesperado);
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
) -> Result<Previa, ErroAcompanhar> {
    if pedido.deixar {
        if pedido.observacao.is_some() {
            return Err(ErroAcompanhar::PedidoDeAcompanhamentoInvalido);
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
    let grupos = grupos.ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
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

/// Fotografia e submissão opcional, sempre derivada da mesma tela.
struct LeituraDoGerenciador {
    antes: Vec<Acompanhamento>,
    url_cadastro: Option<String>,
    exclusao: Option<SubmissaoDerivada>,
}

/// Lê uma única fotografia e prepara exclusão só da linha solicitada.
fn ler_estado_gerenciador<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    instalacao: &Instalacao,
    link: &str,
    cookie: &str,
    id: &str,
    unidade: &str,
    pedido: &Pedido<'_>,
) -> Result<LeituraDoGerenciador, ErroAcompanhar> {
    let res = rede.get_sem_seguir(link, cookie)?;
    let mut exclusao = None;
    let (antes, url_cadastro) = if res.resposta.status_code == 302 {
        let url = redirecionamento_do_gerenciar(instalacao, res.localizacao.as_deref(), link, id)?;
        (Vec::new(), Some(url))
    } else {
        let mime = res.resposta.mime.clone();
        let html = decodificar_resposta(res.resposta)?;
        let antes = acompanhamentos_da_lista(&html)?;
        if pedido.deixar && pedido.confirmar {
            let alvo = antes
                .iter()
                .any(|linha| pedido.grupo.is_none_or(|grupo| linha.grupo == grupo));
            if alvo {
                let alvo = alvo_para_deixar(&antes, pedido.grupo)?;
                let destino = super::exclusao::destino_oferecido(instalacao, &html, id, &alvo.id)?;
                let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
                    .ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
                let tela = crate::formulario_web::TelaServida {
                    url: link,
                    html: &html,
                    codificacao,
                };
                exclusao = Some(super::submissao::derivar_exclusao(
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
    Ok(LeituraDoGerenciador {
        antes,
        url_cadastro,
        exclusao,
    })
}

/// Prévia completa sem escrita, no máximo um GET da lista, mais um do cadastro.
pub(crate) fn previa<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &Pedido<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, ErroAcompanhar> {
    if pedido.deixar && pedido.observacao.is_some() {
        return Err(ErroAcompanhar::PedidoDeAcompanhamentoInvalido);
    }
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(ErroAcompanhar::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )?;
    let unidade = aberto.unidade.ok_or(ErroAcompanhar::UnidadeNaoPublicada)?;
    let id = aberto.arvore.processo.id;
    let link = acoes.acompanhar?.ok_or(ErroAcompanhar::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao)?;
    let LeituraDoGerenciador {
        antes,
        url_cadastro,
        exclusao,
    } = ler_estado_gerenciador(
        &mut rede,
        &instalacao,
        link.as_str(),
        &cookie,
        &id,
        &unidade,
        pedido,
    )?;
    let mut submissao: Option<SubmissaoDerivada> = None;
    let grupos = if pedido.deixar {
        None
    } else {
        let url = url_cadastro.ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
        let (html, mime) = html_da_tela(&mut rede, &cookie, &url)?;
        let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
            .ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
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
    if pedido.deixar {
        let submissao = exclusao.ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
        return confirmar_remocao(
            &mut rede,
            sessao,
            &instalacao,
            &id,
            link.as_str(),
            &cookie,
            &antes,
            plano,
            submissao,
        );
    }
    let submissao = submissao.ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?;
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
) -> Result<Previa, ErroAcompanhar> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |formulario, cookie| {
        enviado = true;
        rede.post(formulario, cookie)
    });
    if !enviado {
        return Err(ErroAcompanhar::Formulario(
            resposta
                .err()
                .ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?,
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
        return Err(ErroAcompanhar::AcompanhamentoNaoConfirmado);
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
        .ok_or(ErroAcompanhar::AcompanhamentoNaoConfirmado)?;
    plano.id_de_remocao = Some(novo.id.clone());
    plano.estado = EstadoDoAcompanhamento::Incluido;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
mod tests;

#[path = "fluxo/retirada.rs"]
mod retirada;
use retirada::confirmar_remocao;
