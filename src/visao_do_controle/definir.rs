//! `ies unidade colunas --definir todas|LISTA [--confirmar]`: lê o Nível de
//! detalhe pelo link da D, planeja, e só com `--confirmar` salva uma vez e
//! confere por um GET da mesma tela. A tela do Controle fica como foi
//! encontrada. Orçamentos em contracts/cli/unidade-definir-colunas.md.
use super::ErroVisaoDoControle as E;
use super::Visao;
use super::definir_erro::{
    ColunasDefinidas, DesfechoDasColunas, ErroDefinirColunas as D, FalhaDoSalvar,
    VerificacaoDasColunas,
};
use super::envio::{EnvioDoControle, Sessao, TelaAtual};
use super::nivel::{link_do_nivel_de_detalhe, obter_tela_do_nivel};
use super::passos::{conferir_resumida_restaurada, contexto, trocar, unidade_da_tela};
use crate::formulario_web::{
    RespostaDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::nivel_de_detalhe::{
    AlvoDasColunas, ColunaDoDetalhe, ColunaIndisponivelNestaInstalacao, PlanoDasColunas,
    SALVAR_NIVEL_DE_DETALHE, campos_do_envio, conferir_controles_do_formulario,
    ler_nivel_de_detalhe, planejar,
};
use crate::pagina_do_controle::codificacao_da_tela;
use std::path::Path;

/// Sem rede antes da tela do Controle já obtida; nenhum envio é repetido.
pub(crate) fn definir_colunas<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaAtual,
    unidade_solicitada: Option<&str>,
    alvo: &AlvoDasColunas,
    confirmar: bool,
    cookie_file: &Path,
    envio: &mut V,
) -> DesfechoDasColunas {
    let sem_volta = |resultado| DesfechoDasColunas {
        resultado,
        visao_nao_restaurada: None,
    };
    let encontrado = match contexto(&tela.html) {
        Ok(encontrado) => encontrado,
        Err(erro) => return sem_volta(Err(D::Leitura(erro))),
    };
    let unidade = match unidade_da_tela(&encontrado, unidade_solicitada) {
        Ok(unidade) => unidade,
        Err(erro) => return sem_volta(Err(D::Leitura(erro))),
    };
    let mut sessao = Sessao {
        instalacao,
        unidade: &unidade,
        cookie_file,
        envio,
    };
    let pedido = Pedido { alvo, confirmar };
    if encontrado.eh_detalhada() {
        return sem_volta(na_detalhada(&mut sessao, tela, &pedido));
    }
    if !encontrado.eh_resumida() {
        return sem_volta(Err(D::Leitura(E::VisaoDoControleNaoReconhecida)));
    }
    let detalhada = match trocar(&mut sessao, tela, Visao::Detalhada) {
        Ok(detalhada) => detalhada,
        Err(erro) => return sem_volta(Err(D::Leitura(erro))),
    };
    let resultado = na_detalhada(&mut sessao, &detalhada, &pedido);
    let volta = trocar(&mut sessao, &detalhada, Visao::Resumida)
        .and_then(|restaurada| conferir_resumida_restaurada(&tela.html, &restaurada.html))
        .err()
        .map(|causa| E::VisaoDoControleNaoRestaurada {
            visao_deixada: Visao::Detalhada,
            causa: Box::new(causa),
        });
    DesfechoDasColunas {
        resultado,
        visao_nao_restaurada: volta,
    }
}

struct Pedido<'a> {
    alvo: &'a AlvoDasColunas,
    confirmar: bool,
}

/// A partir de uma tela D: GET do Nível de detalhe, plano, derivação e,
/// confirmado, o POST de salvar e o GET de verificação.
fn na_detalhada<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    detalhada: &TelaAtual,
    pedido: &Pedido<'_>,
) -> Result<ColunasDefinidas, D> {
    let url = link_do_nivel_de_detalhe(sessao.instalacao, detalhada).map_err(D::Leitura)?;
    let servida = obter_tela_do_nivel(sessao, &url).map_err(D::Leitura)?;
    let nivel =
        ler_nivel_de_detalhe(&servida.html).map_err(|erro| D::Leitura(E::NivelDeDetalhe(erro)))?;
    conferir_controles_do_formulario(&servida.html)
        .map_err(|erro| D::Leitura(E::NivelDeDetalhe(erro)))?;
    let plano =
        planejar(&nivel, pedido.alvo).map_err(|ColunaIndisponivelNestaInstalacao { coluna }| {
            D::ColunaIndisponivelNestaInstalacao { coluna }
        })?;
    let mut feito = ColunasDefinidas {
        unidade: sessao.unidade.to_owned(),
        plano,
        confirmado: pedido.confirmar,
        alterado: false,
        campos_do_envio: None,
        depois: None,
        reconciliado_por: None,
    };
    if feito.plano.inalterado() {
        return Ok(feito);
    }
    let submissao = derivar_salvar(sessao, &servida, &feito.plano).map_err(D::Leitura)?;
    feito.campos_do_envio = Some(submissao.nomes_dos_campos().to_vec());
    if !pedido.confirmar {
        return Ok(feito);
    }
    let falha = salvar(sessao, submissao)?;
    let verificacao = match obter_tela_do_nivel(sessao, &url)
        .and_then(|lida| ler_nivel_de_detalhe(&lida.html).map_err(E::NivelDeDetalhe))
    {
        Ok(lido) => VerificacaoDasColunas::Marcadas(lido.marcadas()),
        Err(erro) => VerificacaoDasColunas::LeituraFalhou(erro),
    };
    conciliar(feito, falha, verificacao)
}

fn derivar_salvar<V: EnvioDoControle>(
    sessao: &Sessao<'_, V>,
    servida: &TelaAtual,
    plano: &PlanoDasColunas,
) -> Result<SubmissaoDerivada, E> {
    let codificacao = codificacao_da_tela(&servida.mime).ok_or(E::CharsetDaTelaNaoSuportado)?;
    let tela = TelaServida {
        url: &servida.url,
        html: &servida.html,
        codificacao,
    };
    derivar_submissao(
        &SALVAR_NIVEL_DE_DETALHE,
        sessao.instalacao,
        &tela,
        sessao.unidade,
        &campos_do_envio(plano),
    )
    .map_err(E::Submissao)
}

/// Um único POST. `Ok(None)`: resposta esperada; `Ok(Some(falha))`: o
/// pedido saiu e a resposta não é a esperada; `Err`: nada saiu.
fn salvar<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    submissao: SubmissaoDerivada,
) -> Result<Option<FalhaDoSalvar>, D> {
    let mut enviado = false;
    let envio = &mut *sessao.envio;
    let resposta = enviar_submissao_via(submissao, sessao.cookie_file, |s, c| {
        enviado = true;
        envio.post(s, c)
    });
    match resposta {
        Ok(RespostaDoEnvio::Html(_)) => Ok(None),
        Ok(RespostaDoEnvio::Redirecionamento(_)) => Ok(Some(FalhaDoSalvar::Redirecionamento302)),
        Err(erro) if enviado => Ok(Some(FalhaDoSalvar::Envio(erro))),
        Err(erro) => Err(D::Leitura(E::Submissao(erro))),
    }
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    mut feito: ColunasDefinidas,
    falha: Option<FalhaDoSalvar>,
    verificacao: VerificacaoDasColunas,
) -> Result<ColunasDefinidas, D> {
    let plano = Box::new(feito.plano.clone());
    let observadas: Vec<ColunaDoDetalhe> = match verificacao {
        VerificacaoDasColunas::Marcadas(ref marcadas) => marcadas.clone(),
        VerificacaoDasColunas::LeituraFalhou(_) => {
            return Err(D::EfeitoIncertoNaConfiguracaoDeColunas {
                plano,
                falha_do_salvar: falha,
                verificacao: Box::new(verificacao),
            });
        }
    };
    if observadas == plano.alvo {
        feito.alterado = true;
        feito.depois = Some(observadas);
        feito.reconciliado_por = falha;
        return Ok(feito);
    }
    if observadas == plano.antes {
        return Err(D::ConfiguracaoDeColunasNaoAplicada {
            plano,
            falha_do_salvar: falha,
        });
    }
    match falha {
        None => Err(D::MarcasDiferentesDoAlvoAposSalvar { plano, observadas }),
        Some(falha) => Err(D::EfeitoIncertoNaConfiguracaoDeColunas {
            plano,
            falha_do_salvar: Some(falha),
            verificacao: Box::new(verificacao),
        }),
    }
}
