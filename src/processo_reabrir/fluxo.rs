//! A sequência de `ies processo reabrir` (contracts/cli/processo-reabrir.md):
//! abertura, decisão antes de escrever, a tela do nó, uma escrita e a
//! leitura de verificação, com a tabela de reconciliação do contrato.
//! Nenhum 302 é seguido; nenhuma escrita é repetida. As peças são as de
//! `processo concluir` (`processo_concluir::escrita`).
use super::erro::ErroReabrir as R;
use super::{PedidoDaReabertura, ProcessoReaberto};
use crate::arvore_processo::ler_endereco_de_reabrir;
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
) -> Result<ProcessoReaberto, R> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(R::Abertura(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let mut rede = Contador::novo(envio);
    let leitura = |unidade_exigida| leitura(pedido.nup, pedido.unidade, unidade_exigida);
    let (aberto, acoes) = abrir_para_escrever(sessao, &leitura(true), indice, &mut rede)?;
    let unidade = aberto
        .unidade
        .clone()
        .ok_or(R::UnidadeDaSessaoNaoPublicada)?;
    let aberto_em = siglas_abertas(&aberto.arvore.aberto_em)
        .map_err(|motivo| R::SituacaoDoProcessoNaoLidaAntesDeReabrir { motivo })?;
    let na_unidade = aberto_em.contains(&unidade);
    let mut feito = ProcessoReaberto {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: aberto.arvore.processo.id.clone(),
        unidade,
        confirmado: pedido.confirmar,
        alterado: false,
        oferecido: false,
        forma: None,
        aberto_em,
        depois: None,
        reconciliado_por: None,
        requisicoes: rede.feitas(),
    };
    let acao = match (acoes.reabrir?, na_unidade) {
        (None, true) => return Ok(feito),
        (Some(_), true) => {
            return Err(R::UnidadeDaSessaoJaNaSituacao {
                aberto_em: feito.aberto_em,
            });
        }
        (None, false) => {
            return Err(R::ReaberturaNaoPublicadaNaBarra {
                aberto_em: feito.aberto_em,
            });
        }
        (Some(acao), false) => acao,
    };
    feito.oferecido = true;
    if !pedido.confirmar {
        return Ok(feito);
    }
    let cookie = ler_cookie(sessao)?;
    let id = feito.id_procedimento.clone();
    let (html, _) =
        obter_tela(&mut rede, &cookie, acao.tela_do_no.as_str()).map_err(|falha| match falha {
            FalhaDaTela::Transporte(falha) => R::TelaDoNo(falha),
            FalhaDaTela::Redirecionada => R::TelaDoNoRedirecionada,
            FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
                R::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
            }
        })?;
    let (url, forma) = ler_endereco_de_reabrir(&instalacao, &html, &id)?;
    feito.forma = Some(forma);
    let falha = pela_funcao(
        &mut rede,
        &cookie,
        url.as_str(),
        FalhaDoEnvio::RedirecionamentoNaReabertura,
    );
    let verificacao = verificar(sessao, &leitura(false), indice, &mut rede);
    feito.requisicoes = rede.feitas();
    conciliar(feito, falha, verificacao)
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    mut feito: ProcessoReaberto,
    falha: Option<FalhaDoEnvio>,
    verificacao: VerificacaoPorLeitura,
) -> Result<ProcessoReaberto, R> {
    let VerificacaoPorLeitura::Lida(depois) = verificacao else {
        return Err(R::NaoConfirmadoNaReabertura {
            aberto_em: feito.aberto_em,
            falha_do_envio: falha,
            verificacao: Box::new(verificacao),
        });
    };
    if !depois.contains(&feito.unidade) {
        return Err(R::ReaberturaNaoAplicada {
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
