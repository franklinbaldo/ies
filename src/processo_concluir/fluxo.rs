//! A sequência de `ies processo concluir` (contracts/cli/processo-concluir.md):
//! abertura, decisão antes de escrever, uma escrita e a leitura de
//! verificação, com a tabela de reconciliação do contrato. Nenhum 302 é
//! seguido; nenhuma escrita é repetida.
use super::erro::{ErroConcluir as C, FalhaDoEnvio, VerificacaoPorLeitura};
use super::escrita::{
    Contador, FalhaDaTela, leitura, ler_cookie, obter_tela, pela_funcao, siglas_abertas, verificar,
};
use super::tela::{
    CAMPOS_DO_ENVIO, CONCLUIR_PROCESSO, conferir_tela_de_conclusao, redirecionamento_esperado,
};
use super::{FormaDaConclusao, PedidoDaConclusao, ProcessoConcluido};
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
) -> Result<ProcessoConcluido, C> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(C::Abertura(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let mut rede = Contador::novo(envio);
    let leitura = |unidade_exigida| leitura(pedido.nup, pedido.unidade, unidade_exigida);
    let (aberto, acoes) = abrir_para_escrever(sessao, &leitura(true), indice, &mut rede)?;
    let concluir = acoes.concluir;
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
        requisicoes: rede.feitas(),
    };
    let acao = match (concluir?, na_unidade) {
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
            let (html, _) = obter_tela(&mut rede, &cookie, tela_do_no.as_str()).map_err(da_tela)?;
            let url = ler_funcao_de_concluir(&instalacao, &html, &id)?;
            pela_funcao(
                &mut rede,
                &cookie,
                url.as_str(),
                FalhaDoEnvio::RedirecionamentoNaFuncao,
            )
        }
    };
    let verificacao = verificar(sessao, &leitura(false), indice, &mut rede);
    feito.requisicoes = rede.feitas();
    conciliar(feito, falha, verificacao)
}

fn da_tela(falha: FalhaDaTela) -> C {
    match falha {
        FalhaDaTela::Transporte(falha) => C::TelaDaEscrita(falha),
        FalhaDaTela::Redirecionada => C::TelaDaEscritaRedirecionada,
        FalhaDaTela::SessaoEncerrada => C::SessaoEncerrada,
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
    let (html, mime) = obter_tela(rede, cookie, url).map_err(da_tela)?;
    conferir_tela_de_conclusao(&html, id)?;
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
        Err(erro) => Err(C::Formulario(erro)),
    }
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    mut feito: ProcessoConcluido,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<ProcessoConcluido, C> {
    let depois = match verificacao {
        VerificacaoPorLeitura::Lida(ref siglas) => siglas.clone(),
        _ => {
            return Err(C::NaoConfirmadoNaConclusao {
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
