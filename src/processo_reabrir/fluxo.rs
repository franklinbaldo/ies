//! A sequência de `ies processo reabrir` (contracts/cli/processo-reabrir.md):
//! abertura, decisão antes de escrever, a tela do nó, uma escrita e a
//! leitura de verificação, com a tabela de reconciliação do contrato.
//! Nenhum 302 é seguido; nenhuma escrita é repetida. As peças são as de
//! `processo concluir` (`processo_concluir::escrita`).
use super::{DesfechoDaReabertura, PedidoDaReabertura, ProcessoReaberto};
use crate::arvore_processo::{FormaDaReabertura, ler_endereco_de_reabrir};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo_concluir::escrita::{
    Contador, FalhaDaTela, leitura, ler_cookie, obter_tela, pela_funcao, siglas_abertas, verificar,
};
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::FalhaTransporte;

/// Sem rede além da abertura na prévia; uma escrita, nunca repetida.
pub(crate) fn reabrir_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaReabertura<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoReaberto, super::erro::ErroReabrir> {
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(super::erro::ErroReabrir::Abertura(
            ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
        ))?;
    let mut rede = Contador::novo(envio);
    let leitura = |unidade_exigida| leitura(pedido.nup, pedido.unidade, unidade_exigida);
    let (aberto, acoes) = abrir_para_escrever(sessao, &leitura(true), indice, &mut rede)?;
    let unidade = aberto
        .unidade
        .clone()
        .ok_or(super::erro::ErroReabrir::UnidadeDaSessaoNaoPublicada)?;
    let aberto_em = siglas_abertas(&aberto.arvore.aberto_em).map_err(|motivo| {
        super::erro::ErroReabrir::SituacaoDoProcessoNaoLidaAntesDeReabrir { motivo }
    })?;
    let na_unidade = aberto_em.contains(&unidade);
    let id = aberto.arvore.processo.id;
    let desfecho =
        match (acoes.reabrir?, na_unidade) {
            (None, true) => DesfechoDaReabertura::JaAbertoNaUnidade {
                confirmado: pedido.confirmar,
            },
            (Some(_), true) => {
                return Err(super::erro::ErroReabrir::UnidadeDaSessaoJaNaSituacao { aberto_em });
            }
            (None, false) => {
                return Err(super::erro::ErroReabrir::ReaberturaNaoPublicadaNaBarra { aberto_em });
            }
            (Some(acao), false) => {
                if pedido.confirmar {
                    let cookie = ler_cookie(sessao)?;
                    let (html, _) = obter_tela(&mut rede, &cookie, acao.tela_do_no.as_str())
                        .map_err(|falha| match falha {
                            FalhaDaTela::Transporte(falha) => {
                                super::erro::ErroReabrir::TelaDoNo(falha)
                            }
                            FalhaDaTela::Redirecionada => {
                                super::erro::ErroReabrir::TelaDoNoRedirecionada
                            }
                            FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
                                super::erro::ErroReabrir::SessaoEncerrada(
                                    crate::erro_publico::SessaoEncerrada,
                                )
                            }
                        })?;
                    let (url, forma) = ler_endereco_de_reabrir(&instalacao, &html, &id)?;
                    let falha = pela_funcao(
                        &mut rede,
                        &cookie,
                        url.as_str(),
                        FalhaDoEnvio::RedirecionamentoNaReabertura,
                    );
                    let verificacao = verificar(sessao, &leitura(false), indice, &mut rede);
                    conciliar(forma, &aberto_em, &unidade, falha, verificacao)?
                } else {
                    DesfechoDaReabertura::Previa
                }
            }
        };
    Ok(ProcessoReaberto {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: id,
        unidade,
        aberto_em,
        requisicoes: rede.feitas(),
        desfecho,
    })
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    forma: FormaDaReabertura,
    aberto_em: &[String],
    unidade: &str,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<DesfechoDaReabertura, super::erro::ErroReabrir> {
    let VerificacaoPorLeitura::Lida(depois) = verificacao else {
        return Err(super::erro::ErroReabrir::NaoConfirmadoNaReabertura {
            aberto_em: aberto_em.to_vec(),
            falha_do_envio: falha,
            verificacao: Box::new(verificacao),
        });
    };
    if !depois.iter().any(|sigla| sigla == unidade) {
        return Err(super::erro::ErroReabrir::ReaberturaNaoAplicada {
            aberto_em: aberto_em.to_vec(),
            depois,
            falha_do_envio: falha,
        });
    }
    Ok(DesfechoDaReabertura::Reaberto {
        forma,
        depois,
        reconciliado_por: falha,
    })
}
