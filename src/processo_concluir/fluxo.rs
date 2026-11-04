//! A sequência de `ies processo concluir` (contracts/cli/processo-concluir.md):
//! abertura, decisão antes de escrever, uma escrita e a leitura de
//! verificação, com a tabela de reconciliação do contrato. Nenhum 302 é
//! seguido; nenhuma escrita é repetida.
use super::erro::{ErroConcluir as C, FalhaDoEnvio, VerificacaoDaConclusao};
use super::tela::{
    CAMPOS_DO_ENVIO, CONCLUIR_PROCESSO, conferir_tela_de_conclusao, redirecionamento_esperado,
};
use super::{FormaDaConclusao, PedidoDaConclusao, ProcessoConcluido};
use crate::arvore_processo::{AbertoEm, ConcluirNaBarra, MotivoAbertoEm, ler_funcao_de_concluir};
use crate::formulario_web::{
    ErroSubmissaoFormulario, RespostaDoEnvio, RespostaDoPost, SubmissaoDerivada, TelaServida,
    derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    abrir_para_concluir, obter_processo,
};
use crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra;
use crate::transporte::{FalhaTransporte, RespostaWeb, StatusHttpRecusado, decodificar_resposta};
use crate::visao_do_controle::EnvioDoControle;

/// Sem rede além da abertura na prévia; uma escrita, nunca repetida.
pub(crate) fn concluir_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaConclusao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoConcluido, C> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(C::Abertura(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let mut rede = Contador { envio, feitas: 0 };
    let leitura = |unidade_exigida| PedidoDoProcesso {
        nup: pedido.nup,
        unidade: pedido.unidade,
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: false,
        unidade_exigida,
    };
    let (aberto, concluir) =
        abrir_para_concluir(sessao, &leitura(true), indice, &mut rede).map_err(C::Abertura)?;
    let unidade = aberto
        .unidade
        .clone()
        .ok_or(C::UnidadeDaSessaoNaoPublicada)?;
    let aberto_em = siglas_abertas(&aberto.arvore.aberto_em)
        .map_err(|motivo| C::SituacaoDoProcessoNaoLidaAntesDeConcluir { motivo })?;
    let na_unidade = aberto_em.contains(&unidade);
    let mut feito = ProcessoConcluido {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: aberto.arvore.processo.id.clone(),
        unidade,
        confirmado: pedido.confirmar,
        alterado: false,
        forma: None,
        aberto_em,
        depois: None,
        reconciliado_por: None,
        requisicoes: rede.feitas,
    };
    let acao = match (concluir.map_err(C::Barra)?, na_unidade) {
        (None, false) => return Ok(feito),
        (Some(_), false) => {
            return Err(C::UnidadeDaSessaoAusenteDaSituacao {
                aberto_em: feito.aberto_em,
            });
        }
        (None, true) => {
            return Err(C::ConclusaoNaoPublicadaNaBarra {
                aberto_em: feito.aberto_em,
            });
        }
        (Some(acao), true) => acao,
    };
    feito.forma = Some(match acao {
        ConcluirNaBarra::PelaTela(_) => FormaDaConclusao::Tela,
        ConcluirNaBarra::PelaFuncao { .. } => FormaDaConclusao::Funcao,
    });
    if !pedido.confirmar {
        return Ok(feito);
    }
    let cookie = ler_cookie(sessao)?;
    let id = feito.id_procedimento.clone();
    let falha = match acao {
        ConcluirNaBarra::PelaTela(url) => {
            let submissao = derivar_pela_tela(
                &mut rede,
                &cookie,
                &instalacao,
                url.as_str(),
                &id,
                &feito.unidade,
            )?;
            enviar(&mut rede, sessao, submissao, &id)?
        }
        ConcluirNaBarra::PelaFuncao { tela_do_no } => {
            let (html, _) = obter_tela(&mut rede, &cookie, tela_do_no.as_str())?;
            let url =
                ler_funcao_de_concluir(&instalacao, &html, &id).map_err(C::FuncaoDeConcluir)?;
            pela_funcao(&mut rede, &cookie, url.as_str())
        }
    };
    let verificacao = match obter_processo(sessao, &leitura(false), indice, &mut rede) {
        Ok(lido) => match siglas_abertas(&lido.arvore.aberto_em) {
            Ok(siglas) => VerificacaoDaConclusao::Lida(siglas),
            Err(motivo) => VerificacaoDaConclusao::SituacaoNaoLida(motivo),
        },
        Err(erro) => VerificacaoDaConclusao::LeituraFalhou(erro),
    };
    feito.requisicoes = rede.feitas;
    conciliar(feito, falha, verificacao)
}

/// As siglas de `aberto_em`, ou o motivo de não ter sido lido.
fn siglas_abertas(aberto_em: &AbertoEm) -> Result<Vec<String>, MotivoAbertoEm> {
    match aberto_em {
        AbertoEm::Unidades(unidades) => Ok(unidades.iter().map(|u| u.sigla.clone()).collect()),
        AbertoEm::Sobrestado(_) => Err(MotivoAbertoEm::RaizPublicaOSobrestamento),
        AbertoEm::NaoLido(motivo) => Err(*motivo),
    }
}

fn ler_cookie(sessao: &SessaoDoProcesso<'_>) -> Result<String, C> {
    let falha = |f| C::Formulario(ErroSubmissaoFormulario::Transporte(f));
    crate::transporte::sessao::cookie_privado_validado(sessao.cookie_file).map_err(falha)
}

/// GET de uma tela antes da escrita (a de conclusão ou a do nó): HTML 200.
fn obter_tela<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
) -> Result<(String, String), C> {
    let resposta = rede.get_sem_seguir(url, cookie).map_err(C::TelaDaEscrita)?;
    if resposta.resposta.status_code == 302 {
        return Err(C::TelaDaEscritaRedirecionada);
    }
    let mime = resposta.resposta.mime.clone();
    match decodificar_resposta(resposta.resposta) {
        Ok(html) => Ok((html, mime)),
        Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle) => Err(C::SessaoEncerrada),
        Err(falha) => Err(C::TelaDaEscrita(falha)),
    }
}

/// Forma `tela`: GET da tela, conferência e derivação do POST (sem rede).
fn derivar_pela_tela<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    instalacao: &Instalacao,
    url: &str,
    id: &str,
    unidade: &str,
) -> Result<SubmissaoDerivada, C> {
    let (html, mime) = obter_tela(rede, cookie, url)?;
    conferir_tela_de_conclusao(&html, id).map_err(C::TelaDeConclusao)?;
    let codificacao = codificacao_da_tela(&mime).ok_or(C::CharsetDaTelaNaoSuportado)?;
    let tela = TelaServida {
        url,
        html: &html,
        codificacao,
    };
    derivar_submissao(
        &CONCLUIR_PROCESSO,
        instalacao,
        &tela,
        unidade,
        &CAMPOS_DO_ENVIO,
    )
    .map_err(C::Formulario)
}

/// Um único POST. `Ok(None)`: o 302 esperado; `Ok(Some(falha))`: o pedido
/// saiu e a resposta não é a esperada; `Err`: nada saiu.
fn enviar<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    submissao: SubmissaoDerivada,
    id: &str,
) -> Result<Option<FalhaDoEnvio>, C> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie_file, |s, c| {
        enviado = true;
        rede.post(s, c)
    });
    match resposta {
        Ok(RespostaDoEnvio::Redirecionamento(r)) => {
            Ok((!redirecionamento_esperado(r.localizacao(), id))
                .then_some(FalhaDoEnvio::RedirecionamentoNaoReconhecido))
        }
        Ok(RespostaDoEnvio::Html(_)) => Ok(Some(FalhaDoEnvio::RespostaSemRedirecionamento)),
        Err(erro) if enviado => Ok(Some(FalhaDoEnvio::Envio(erro))),
        Err(erro) => Err(C::Formulario(erro)),
    }
}

/// Forma `funcao`: o GET da URL da função é a escrita. `None`: HTML 200
/// sem login nem validação da Infra.
fn pela_funcao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
) -> Option<FalhaDoEnvio> {
    let resposta = match rede.get_sem_seguir(url, cookie) {
        Ok(resposta) => resposta,
        Err(falha) => return Some(FalhaDoEnvio::Transporte(falha)),
    };
    if resposta.resposta.status_code == 302 {
        return Some(FalhaDoEnvio::RedirecionamentoNaFuncao);
    }
    match decodificar_resposta(resposta.resposta) {
        Ok(html) => mensagem_de_validacao_da_infra(&html).map(|mensagem| {
            FalhaDoEnvio::Envio(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(
                StatusHttpRecusado {
                    status: 200,
                    mensagem_do_servidor: Some(mensagem),
                },
            ))
        }),
        Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle) => {
            Some(FalhaDoEnvio::LoginAposEscrita)
        }
        Err(falha) => Some(FalhaDoEnvio::Transporte(falha)),
    }
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    mut feito: ProcessoConcluido,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoDaConclusao,
) -> Result<ProcessoConcluido, C> {
    let depois = match verificacao {
        VerificacaoDaConclusao::Lida(ref siglas) => siglas.clone(),
        _ => {
            return Err(C::EfeitoIncertoNaConclusao {
                aberto_em: feito.aberto_em,
                falha_do_envio: falha,
                verificacao: Box::new(verificacao),
            });
        }
    };
    if depois.contains(&feito.unidade) {
        return Err(C::ConclusaoNaoAplicada {
            aberto_em: feito.aberto_em,
            depois,
            falha_do_envio: falha,
        });
    }
    feito.alterado = true;
    feito.depois = Some(depois);
    feito.reconciliado_por = falha;
    Ok(feito)
}

/// O transporte com a contagem de todas as requisições do comando
/// (inclusive as que falharam), da abertura à verificação.
struct Contador<'v, V> {
    envio: &'v mut V,
    feitas: u8,
}

impl<V: EnvioDoProcesso> EnvioDoControle for Contador<'_, V> {
    type Corpo = V::Corpo;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.feitas = self.feitas.saturating_add(1);
        self.envio.post(submissao, cookie)
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        self.feitas = self.feitas.saturating_add(1);
        self.envio.get(url, cookie)
    }
    fn colher_controle(&mut self, html: &str, mime: Option<&str>) {
        self.envio.colher_controle(html, mime);
    }
}

impl<V: EnvioDoProcesso> EnvioDoProcesso for Contador<'_, V> {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.feitas = self.feitas.saturating_add(1);
        self.envio.get_sem_seguir(url, cookie)
    }
    fn get_binario(
        &mut self,
        url: &str,
        cookie: &str,
        limite: usize,
    ) -> Result<crate::transporte::resposta_binaria::RespostaBinaria<Self::Corpo>, FalhaTransporte>
    {
        self.feitas = self.feitas.saturating_add(1);
        self.envio.get_binario(url, cookie, limite)
    }
}
