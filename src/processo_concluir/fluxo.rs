//! A sequência de `ies processo concluir` (contracts/cli/processo-concluir.md):
//! abertura, decisão antes de escrever, uma escrita e a leitura de
//! verificação, com a tabela de reconciliação do contrato. Nenhum 302 é
//! seguido; nenhuma escrita é repetida.
use super::erro::{ErroConcluir, FalhaDoEnvio, VerificacaoPorLeitura};
use super::escrita::{
    Contador, FalhaDaTela, leitura, ler_cookie, obter_tela, pela_funcao, siglas_abertas, verificar,
};
use super::tela::{
    CAMPOS_DO_ENVIO, CONCLUIR_PROCESSO, conferir_tela_de_conclusao, redirecionamento_esperado,
};
use super::{DesfechoDaConclusao, FormaDaConclusao, PedidoDaConclusao, ProcessoConcluido};
use crate::arvore_processo::{ConcluirNaBarra, ler_funcao_de_concluir};
use crate::formulario_web::{
    RespostaDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::FalhaTransporte;
use crate::visao_do_controle::EnvioDoControle as _;

/// Sem rede além da abertura na prévia; uma escrita, nunca repetida.
pub(crate) fn concluir_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaConclusao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoConcluido, ErroConcluir> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(ErroConcluir::Abertura(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let mut rede = Contador::novo(envio);
    let leitura = |unidade_exigida| leitura(pedido.nup, pedido.unidade, unidade_exigida);
    let (aberto, acoes) = abrir_para_escrever(sessao, &leitura(true), indice, &mut rede)?;
    let concluir = acoes.concluir;
    let unidade = aberto
        .unidade
        .clone()
        .ok_or(ErroConcluir::UnidadeDaSessaoNaoPublicada)?;
    let aberto_em = siglas_abertas(&aberto.arvore.aberto_em)
        .map_err(|motivo| ErroConcluir::SituacaoDoProcessoNaoLidaAntesDeConcluir { motivo })?;
    let na_unidade = aberto_em.contains(&unidade);
    let id = aberto.arvore.processo.id.clone();
    let desfecho = match (concluir?, na_unidade) {
        (None, false) => DesfechoDaConclusao::NaoAbertoNaUnidade {
            confirmado: pedido.confirmar,
        },
        (Some(_), false) => {
            return Err(ErroConcluir::UnidadeDaSessaoAusenteDaSituacao { aberto_em });
        }
        (None, true) => {
            return Err(ErroConcluir::ConclusaoNaoPublicadaNaBarra { aberto_em });
        }
        (Some(acao), true) => {
            let forma = match &acao {
                ConcluirNaBarra::PelaTela(_) => FormaDaConclusao::Tela,
                ConcluirNaBarra::PelaFuncao { .. } => FormaDaConclusao::Funcao,
            };
            if pedido.confirmar {
                let cookie = ler_cookie(sessao)?;
                let falha = escrever(&mut rede, sessao, &instalacao, &cookie, acao, &id, &unidade)?;
                let verificacao = verificar(sessao, &leitura(false), indice, &mut rede);
                conciliar(forma, &aberto_em, &unidade, falha, verificacao)?
            } else {
                DesfechoDaConclusao::Previa { forma }
            }
        }
    };
    Ok(ProcessoConcluido {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: id,
        unidade,
        aberto_em,
        requisicoes: rede.feitas(),
        desfecho,
    })
}

/// O único POST (forma `tela`) ou o GET que é a escrita (forma `funcao`).
/// `Ok(None)`: a resposta esperada; `Ok(Some(falha))`: o pedido saiu e a
/// resposta não é a esperada; `Err`: nada saiu.
fn escrever<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    acao: ConcluirNaBarra,
    id: &str,
    unidade: &str,
) -> Result<Option<FalhaDoEnvio>, ErroConcluir> {
    match acao {
        ConcluirNaBarra::PelaTela(url) => {
            let submissao = derivar_pela_tela(rede, cookie, instalacao, url.as_str(), id, unidade)?;
            enviar(rede, sessao, submissao, id)
        }
        ConcluirNaBarra::PelaFuncao { tela_do_no } => {
            let (html, _) = obter_tela(rede, cookie, tela_do_no.as_str()).map_err(da_tela)?;
            let url = ler_funcao_de_concluir(instalacao, &html, id)?;
            Ok(pela_funcao(
                rede,
                cookie,
                url.as_str(),
                FalhaDoEnvio::RedirecionamentoNaFuncao,
            ))
        }
    }
}

fn da_tela(falha: FalhaDaTela) -> ErroConcluir {
    match falha {
        FalhaDaTela::Transporte(falha) => ErroConcluir::TelaDaEscrita(falha),
        FalhaDaTela::Redirecionada => ErroConcluir::TelaDaEscritaRedirecionada,
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            ErroConcluir::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
        }
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
) -> Result<SubmissaoDerivada, ErroConcluir> {
    let (html, mime) = obter_tela(rede, cookie, url).map_err(da_tela)?;
    conferir_tela_de_conclusao(&html, id)?;
    let codificacao = codificacao_da_tela(&mime).ok_or(ErroConcluir::CharsetDaTelaNaoSuportado)?;
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
    .map_err(ErroConcluir::Formulario)
}

/// Um único POST. `Ok(None)`: o 302 esperado; `Ok(Some(falha))`: o pedido
/// saiu e a resposta não é a esperada; `Err`: nada saiu.
fn enviar<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    submissao: SubmissaoDerivada,
    id: &str,
) -> Result<Option<FalhaDoEnvio>, ErroConcluir> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        enviado = true;
        rede.post(s, c)
    });
    match resposta {
        Ok(RespostaDoEnvio::Redirecionamento(r)) => {
            Ok(
                (!redirecionamento_esperado(r.localizacao(), r.url_do_envio(), id))
                    .then_some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
            )
        }
        Ok(RespostaDoEnvio::Html(_)) => Ok(Some(FalhaDoEnvio::RespostaSemRedirecionamento)),
        Err(erro) if enviado => Ok(Some(FalhaDoEnvio::Envio(erro))),
        Err(erro) => Err(ErroConcluir::Formulario(erro)),
    }
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    forma: FormaDaConclusao,
    aberto_em: &[String],
    unidade: &str,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<DesfechoDaConclusao, ErroConcluir> {
    let depois = match verificacao {
        VerificacaoPorLeitura::Lida(siglas) => siglas,
        nao_lida @ (VerificacaoPorLeitura::SituacaoNaoLida(_)
        | VerificacaoPorLeitura::LeituraFalhou(_)) => {
            return Err(ErroConcluir::NaoConfirmadoNaConclusao {
                aberto_em: aberto_em.to_vec(),
                falha_do_envio: falha,
                verificacao: Box::new(nao_lida),
            });
        }
    };
    if depois.iter().any(|sigla| sigla == unidade) {
        return Err(ErroConcluir::ConclusaoNaoAplicada {
            aberto_em: aberto_em.to_vec(),
            depois,
            falha_do_envio: falha,
        });
    }
    Ok(DesfechoDaConclusao::Concluido {
        forma,
        depois,
        reconciliado_por: falha,
    })
}
