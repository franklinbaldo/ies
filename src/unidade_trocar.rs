//! `ies unidade trocar SIGLA [--confirmar]` (#205;
//! contracts/cli/unidade-trocar.md): a unidade ativa pelo Controle, a tela
//! Trocar Unidade pelo `lnkInfraUnidade`, a escolha da sigla e, com
//! `--confirmar`, um POST e os dois 302 observados até o Controle da
//! unidade nova, que verifica a troca. Sem retry; depois do POST, todo
//! desvio é efeito incerto. A gravação da sessão e o índice ficam com a CLI.
mod erro;
mod retorno;
mod tela;

#[cfg(test)]
mod acervo;
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) mod tests_apoio;
#[cfg(test)]
mod tests_tela;

pub(crate) use erro::{ErroTroca, EtapaDaTroca, FalhaDepoisDoEnvio};
pub(crate) use tela::UnidadeOferecida;

use crate::controle_reconhecido::unidade_do_controle;
use crate::formulario_web::{
    RespostaDoEnvio, RespostaDoPost, SubmissaoDerivada, TelaServida, derivar_submissao,
    enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::FalhaTransporte;
use crate::transporte::decodificar_resposta;
use crate::transporte::sessao::CookieDaSessao;
use retorno::url_do_salto;
use std::io::Read;
use tela::{TROCAR_UNIDADE, campos_do_envio, escolher, ler_tela_de_troca, link_da_troca};

/// Uma requisição por chamada, sem redirect nem retry; o `Location` volta
/// como dado. Os testes roteirizam e conferem a sequência.
pub(crate) trait EnvioDaTroca {
    type Corpo: Read;
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;
}

/// Transporte real: o GET com `Location` e o POST HTTPS dos demais comandos.
pub(crate) struct EnvioHttpsDaTroca;

impl EnvioDaTroca for EnvioHttpsDaTroca {
    type Corpo = crate::transporte::cliente_https::CorpoHttps;
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        crate::transporte::envio_post::get_com_localizacao(url, cookie)
    }
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        crate::transporte::envio_post::enviar_post(submissao, cookie)
    }
}

/// A sessão gravada: URL do Controle (já conferida contra a origem), a
/// instalação e o arquivo de cookie.
pub(crate) struct SessaoDaTroca<'a> {
    pub(crate) url_do_controle: &'a str,
    pub(crate) instalacao: &'a Instalacao,
    pub(crate) cookie: &'a CookieDaSessao,
}

pub(crate) struct PedidoDeTroca<'a> {
    pub(crate) sigla: &'a str,
    pub(crate) confirmar: bool,
    /// `--unidade`: a unidade esperada antes da troca.
    pub(crate) unidade_esperada: Option<&'a str>,
}

/// Uma tela do Controle recebida, para a sessão e a colheita do índice.
pub(crate) struct ControleLido {
    pub(crate) url: String,
    pub(crate) html: String,
    pub(crate) mime: String,
}

/// URL assinada e HTML nunca aparecem no `Debug`.
impl std::fmt::Debug for ControleLido {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ControleLido")
            .field("url", &"<redigido>")
            .field("html", &"<redigido>")
            .field("mime", &self.mime)
            .finish()
    }
}

/// O que o comando leu e, com `--confirmar`, o Controle da unidade nova.
#[derive(Debug)]
pub(crate) struct Troca {
    pub(crate) antes: String,
    pub(crate) pedida: String,
    /// `None` quando a sessão já estava na unidade pedida.
    pub(crate) escolhida: Option<UnidadeOferecida>,
    pub(crate) oferecidas: Vec<UnidadeOferecida>,
    pub(crate) campos_do_envio: Option<Vec<String>>,
    pub(crate) confirmado: bool,
    /// O Controle lido no início (unidade de antes), para a colheita.
    pub(crate) controle_inicial: ControleLido,
    /// O Controle da unidade nova, verificado.
    pub(crate) controle_novo: Option<ControleLido>,
}

/// Sem rede antes de `envio`; nenhum pedido é repetido.
pub(crate) fn trocar_unidade<V: EnvioDaTroca>(
    sessao: &SessaoDaTroca<'_>,
    pedido: &PedidoDeTroca<'_>,
    envio: &mut V,
) -> Result<Troca, ErroTroca> {
    let cookie = sessao.cookie.valor()?;
    let inicial = ler_html(envio, sessao.url_do_controle, cookie)?;
    let antes = unidade_do_controle(&inicial.html)?;
    if pedido.unidade_esperada.is_some_and(|u| u != antes) {
        return Err(ErroTroca::UnidadeExibidaDiferenteDaSolicitada);
    }
    let mut troca = Troca {
        antes: antes.clone(),
        pedida: pedido.sigla.to_owned(),
        escolhida: None,
        oferecidas: Vec::new(),
        campos_do_envio: None,
        confirmado: pedido.confirmar,
        controle_inicial: inicial,
        controle_novo: None,
    };
    if pedido.sigla == antes {
        return Ok(troca);
    }
    let url_da_tela = link_da_troca(
        sessao.instalacao,
        sessao.url_do_controle,
        &troca.controle_inicial.html,
    )?;
    let servida = ler_html(envio, &url_da_tela, cookie)?;
    let tela = ler_tela_de_troca(&servida.html)?;
    if tela.marcada.sigla != antes {
        return Err(ErroTroca::UnidadeMarcadaDiferenteDaAtiva);
    }
    troca.oferecidas.clone_from(&tela.unidades);
    let escolhida = escolher(&tela, pedido.sigla)?.clone();
    let codificacao = codificacao_da_tela(&servida.mime).ok_or(ErroTroca::Transporte(
        FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado(
            crate::transporte::corpo::ErroCorpoHttp::CodificacaoNaoSuportada,
        ),
    ))?;
    let tela_servida = TelaServida {
        url: &servida.url,
        html: &servida.html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &TROCAR_UNIDADE,
        sessao.instalacao,
        &tela_servida,
        &antes,
        &campos_do_envio(&tela, &escolhida),
    )?;
    troca.campos_do_envio = Some(submissao.nomes_dos_campos().to_vec());
    troca.escolhida = Some(escolhida.clone());
    if !pedido.confirmar {
        return Ok(troca);
    }
    troca.controle_novo = Some(enviar_e_verificar(
        sessao, envio, submissao, cookie, &escolhida,
    )?);
    Ok(troca)
}

/// GET de uma tela 200 (outro status é a recusa do transporte).
fn ler_html<V: EnvioDaTroca>(
    envio: &mut V,
    url: &str,
    cookie: &str,
) -> Result<ControleLido, FalhaTransporte> {
    let RespostaDoPost { resposta, .. } = envio.get(url, cookie)?;
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta)?;
    Ok(ControleLido {
        url: url.to_owned(),
        html,
        mime,
    })
}

/// Um POST; depois, os saltos observados, cada um uma vez. Todo desvio é
/// efeito incerto com a etapa.
fn enviar_e_verificar<V: EnvioDaTroca>(
    sessao: &SessaoDaTroca<'_>,
    envio: &mut V,
    submissao: SubmissaoDerivada,
    cookie: &str,
    escolhida: &UnidadeOferecida,
) -> Result<ControleLido, ErroTroca> {
    let incerto = |etapa, falha| ErroTroca::NaoConfirmado { etapa, falha };
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        enviado = true;
        envio.post(s, c)
    });
    let redirecionado = match resposta {
        Ok(RespostaDoEnvio::Redirecionamento(r)) => r,
        Ok(RespostaDoEnvio::Html(_)) => {
            return Err(incerto(
                EtapaDaTroca::Envio,
                FalhaDepoisDoEnvio::RespostaSemRedirecionamento,
            ));
        }
        Err(erro) if enviado => {
            return Err(incerto(
                EtapaDaTroca::Envio,
                FalhaDepoisDoEnvio::Envio(erro),
            ));
        }
        Err(erro) => return Err(ErroTroca::Submissao(erro)),
    };
    let localizacao = redirecionado.localizacao().ok_or(incerto(
        EtapaDaTroca::Envio,
        FalhaDepoisDoEnvio::RedirecionamentoSemLocalizacao,
    ))?;
    let id = escolhida.id_unidade.as_str();
    let mut url = url_do_salto(localizacao, redirecionado.url_do_envio(), id)
        .map_err(|f| incerto(EtapaDaTroca::Envio, f))?;
    let etapas = [
        EtapaDaTroca::PrimeiroRedirecionamento,
        EtapaDaTroca::SegundoRedirecionamento,
    ];
    for (i, etapa) in etapas.into_iter().enumerate() {
        let RespostaDoPost {
            resposta,
            localizacao,
        } = envio
            .get(&url, cookie)
            .map_err(|f| incerto(etapa, FalhaDepoisDoEnvio::Transporte(f)))?;
        match resposta.status_code {
            302 if i == 0 => {
                let localizacao = localizacao.ok_or(incerto(
                    etapa,
                    FalhaDepoisDoEnvio::RedirecionamentoSemLocalizacao,
                ))?;
                url = url_do_salto(&localizacao, &url, id).map_err(|f| incerto(etapa, f))?;
            }
            200 => {
                let mime = resposta.mime.clone();
                let html = decodificar_resposta(resposta).map_err(|f| {
                    incerto(
                        EtapaDaTroca::ControleDaUnidadeNova,
                        FalhaDepoisDoEnvio::Transporte(f),
                    )
                })?;
                return verificar(ControleLido { url, html, mime }, escolhida);
            }
            status @ 300..=399 => return Err(incerto(etapa, FalhaDepoisDoEnvio::Status(status))),
            _ => {
                let falha = decodificar_resposta(resposta).err().map_or(
                    FalhaDepoisDoEnvio::Status(0),
                    FalhaDepoisDoEnvio::Transporte,
                );
                return Err(incerto(etapa, falha));
            }
        }
    }
    unreachable!("o segundo salto devolve ou falha")
}

/// O Controle da unidade nova mostra a sigla pedida.
fn verificar(novo: ControleLido, escolhida: &UnidadeOferecida) -> Result<ControleLido, ErroTroca> {
    let etapa = EtapaDaTroca::ControleDaUnidadeNova;
    let sigla = unidade_do_controle(&novo.html).map_err(|e| ErroTroca::NaoConfirmado {
        etapa,
        falha: FalhaDepoisDoEnvio::ControleNaoReconhecido(e),
    })?;
    if sigla != escolhida.sigla {
        return Err(ErroTroca::NaoConfirmado {
            etapa,
            falha: FalhaDepoisDoEnvio::UnidadeDoControleDiferenteDaPedida,
        });
    }
    Ok(novo)
}
