//! Uma operação de escrita por confirmação, sem redirecionamento seguido,
//! sem retry e com uma leitura nova para verificar o estado do processo.
//! O formulário e a função são extraídos da mesma geração da sessão.
use super::{
    Acao, ErroSobrestamento as E, EstadoDaMudanca, PedidoDaMudanca, Previa, preparar_com_acao, tela,
};
use crate::arvore_processo::{AbertoEm, ler_endereco_de_retomar};
use crate::formulario_web::{
    RespostaDoEnvio, TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::FalhaDoEnvio;
use crate::processo_concluir::escrita::{
    Contador, FalhaDaTela, leitura, ler_cookie, obter_tela, pela_funcao,
};
use crate::processo_obter::{EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, obter_processo};
use crate::transporte::FalhaTransporte;
use crate::visao_do_controle::EnvioDoControle as _;

/// A função também aceita prévia para que teste e comando usem o mesmo
/// contrato. O estado de entrada e a ação são conferidos antes de enviar.
pub(crate) fn executar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaMudanca<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, E> {
    let PedidoDaMudanca {
        nup,
        unidade,
        acao,
        motivo,
        confirmar,
    } = *pedido;
    if acao == Acao::Sobrestar && motivo.is_none_or(|m| m.trim().is_empty()) {
        return Err(E::MotivoVazio);
    }
    let mut rede = Contador::novo(envio);
    let (mut previa, endereco) = preparar_com_acao(sessao, nup, unidade, acao, indice, &mut rede)?;
    if !confirmar {
        return Ok(previa);
    }
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(E::Abertura(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    ))?;
    let cookie = ler_cookie(sessao).map_err(E::TelaDaEscrita)?;
    match acao {
        Acao::Sobrestar => {
            let (html, mime) =
                obter_tela(&mut rede, &cookie, endereco.as_str()).map_err(mapear_tela)?;
            tela::conferir_formulario(&html, &previa.id_procedimento).map_err(E::TelaSobrestar)?;
            let codificacao = codificacao_da_tela(&mime).ok_or(E::CharsetNaoSuportado)?;
            let tela_servida = TelaServida {
                url: endereco.as_str(),
                html: &html,
                codificacao,
            };
            let valores = [
                ("rdoSobrestar", "S"),
                ("txtProcedimentoDestino", ""),
                ("txtIdentificacaoProcedimentoDestino", ""),
                ("txaMotivo", motivo.unwrap_or_default()),
            ];
            let submissao = derivar_submissao(
                &tela::OPERACAO,
                &instalacao,
                &tela_servida,
                &previa.unidade,
                &valores,
            )
            .map_err(E::Formulario)?;
            // A submissão é consumida no POST único. Qualquer falha depois
            // do início do envio segue para a leitura de reconciliação.
            let mut enviado = false;
            let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
                enviado = true;
                rede.post(s, c)
            });
            if !enviado {
                if let Err(e) = resposta {
                    return Err(E::Formulario(e));
                }
                return Err(E::EfeitoIncerto);
            }
            // Um 302 esperado é evidência, nunca prova suficiente. 200,
            // 502 ou resposta perdida exigem a mesma verificação por leitura.
            let _resposta_esperada = matches!(
                resposta,
                Ok(RespostaDoEnvio::Redirecionamento(ref r))
                    if tela::redirecionamento_esperado(
                        r.localizacao(), r.url_do_envio(), &previa.id_procedimento,
                    )
            );
        }
        Acao::Retomar => {
            // A URL da função só existe na tela do nó. A primeira URL
            // da barra é GET de leitura, não a escrita.
            let (html, _) =
                obter_tela(&mut rede, &cookie, endereco.as_str()).map_err(mapear_tela)?;
            let url = ler_endereco_de_retomar(&instalacao, &html, &previa.id_procedimento)
                .map_err(E::EnderecoDeRetomar)?;
            let _falha = pela_funcao(
                &mut rede,
                &cookie,
                url.as_str(),
                FalhaDoEnvio::RedirecionamentoNaFuncao,
            );
        }
    }
    let verificado = obter_processo(sessao, &leitura(nup, unidade, false), indice, &mut rede);
    previa.requisicoes = rede.feitas();
    let depois = verificado.map_err(|_| E::EfeitoIncerto)?.arvore.aberto_em;
    let aplicado = match (acao, depois) {
        (Acao::Sobrestar, AbertoEm::Sobrestado(unidades)) => {
            unidades.iter().any(|u| u.sigla == previa.unidade)
        }
        (Acao::Retomar, AbertoEm::Unidades(unidades)) => {
            unidades.iter().any(|u| u.sigla == previa.unidade)
        }
        (_, AbertoEm::NaoLido(_)) => return Err(E::EfeitoIncerto),
        _ => false,
    };
    if !aplicado {
        return Err(E::EscritaNaoAplicada);
    }
    previa.estado = EstadoDaMudanca::Aplicada;
    Ok(previa)
}

fn mapear_tela(falha: FalhaDaTela) -> E {
    match falha {
        FalhaDaTela::Transporte(e) => E::TelaDaEscrita(e),
        FalhaDaTela::Redirecionada => E::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada => E::SessaoEncerrada,
    }
}
