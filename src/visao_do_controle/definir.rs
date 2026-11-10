//! `ies unidade colunas --definir todas|LISTA [--confirmar]`: lê o Nível de
//! detalhe pelo link da D, planeja, e só com `--confirmar` salva uma vez e
//! confere por um GET da mesma tela. Vinda da R, a sessão fica na D.
//! Orçamentos em contracts/cli/unidade-definir-colunas.md.
use super::ErroVisaoDoControle as E;
use super::Visao;
use super::definir_erro::{
    ColunasDefinidas, ErroDefinirColunas as D, EstadoDasColunas, FalhaDoSalvar,
    VerificacaoDasColunas,
};
use super::envio::{EnvioDoControle, Sessao, TelaAtual};
use super::nivel::{link_do_nivel_de_detalhe, obter_tela_do_nivel};
use super::passos::{contexto, trocar, unidade_da_tela};
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    RespostaDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::nivel_de_detalhe::{
    AlvoDasColunas, ColunaDoDetalhe, ColunaIndisponivelNestaInstalacao, PlanoDasColunas,
    SALVAR_NIVEL_DE_DETALHE, SinalDoSalvar, campos_do_envio, conferir_controles_do_formulario,
    ler_nivel_de_detalhe, planejar, sinal_do_salvar,
};
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::sessao::CookieDaSessao;

/// Sem rede antes da tela do Controle já obtida; nenhum envio é repetido.
/// Vinda da R, troca para D e fica nela.
pub(crate) fn definir_colunas<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaAtual,
    unidade_solicitada: Option<&str>,
    alvo: &AlvoDasColunas,
    confirmar: bool,
    cookie: &CookieDaSessao,
    envio: &mut V,
) -> Result<ColunasDefinidas, D> {
    let encontrado = contexto(&tela.html)?;
    let unidade = unidade_da_tela(&encontrado, unidade_solicitada)?;
    let mut sessao = Sessao {
        instalacao,
        unidade: &unidade,
        cookie,
        envio,
    };
    let pedido = Pedido { alvo, confirmar };
    if encontrado.eh_detalhada() {
        return na_detalhada(&mut sessao, tela, &pedido);
    }
    if !encontrado.eh_resumida() {
        return Err(D::Leitura(E::Html(ErroHtml::valor(
            "visao_do_controle_nao_reconhecida",
        ))));
    }
    let detalhada = trocar(&mut sessao, tela, Visao::Detalhada)?;
    na_detalhada(&mut sessao, &detalhada, &pedido)
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
    let url = link_do_nivel_de_detalhe(sessao.instalacao, detalhada)?;
    let servida = obter_tela_do_nivel(sessao, &url)?;
    let nivel =
        ler_nivel_de_detalhe(&servida.html).map_err(|erro| D::Leitura(E::NivelDeDetalhe(erro)))?;
    conferir_controles_do_formulario(&servida.html)
        .map_err(|erro| D::Leitura(E::NivelDeDetalhe(erro)))?;
    let plano =
        planejar(&nivel, pedido.alvo).map_err(|ColunaIndisponivelNestaInstalacao { coluna }| {
            D::ColunaIndisponivelNestaInstalacao { coluna }
        })?;
    let unidade = sessao.unidade.to_owned();
    if plano.inalterado() {
        return Ok(ColunasDefinidas {
            unidade,
            plano,
            estado: EstadoDasColunas::Inalteradas {
                confirmado: pedido.confirmar,
            },
        });
    }
    let submissao = derivar_salvar(sessao, &servida, &plano)?;
    let campos_do_envio = submissao.nomes_dos_campos().to_vec();
    if !pedido.confirmar {
        return Ok(ColunasDefinidas {
            unidade,
            plano,
            estado: EstadoDasColunas::Previa { campos_do_envio },
        });
    }
    let falha = salvar(sessao, submissao)?;
    let verificacao = match obter_tela_do_nivel(sessao, &url)
        .and_then(|lida| ler_nivel_de_detalhe(&lida.html).map_err(E::NivelDeDetalhe))
    {
        Ok(lido) => VerificacaoDasColunas::Marcadas(lido.marcadas()),
        Err(erro) => VerificacaoDasColunas::LeituraFalhou(erro),
    };
    conciliar(unidade, plano, campos_do_envio, falha, verificacao)
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

/// Um único POST. `Ok(None)`: resposta esperada (HTML com o sinal de
/// gravação); `Ok(Some(falha))`: o pedido saiu e a resposta não é a
/// esperada; `Err`: nada saiu.
fn salvar<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    submissao: SubmissaoDerivada,
) -> Result<Option<FalhaDoSalvar>, D> {
    let mut enviado = false;
    let envio = &mut *sessao.envio;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        enviado = true;
        envio.post(s, c)
    });
    match resposta {
        Ok(RespostaDoEnvio::Html(html)) => Ok(match sinal_do_salvar(&html) {
            SinalDoSalvar::GravacaoSinalizada => None,
            SinalDoSalvar::TelaDevolvidaSemSinal => {
                Some(FalhaDoSalvar::NivelDevolvidoSemSinalDeGravacao)
            }
            SinalDoSalvar::NaoReconhecido => Some(FalhaDoSalvar::RespostaDoSalvarNaoReconhecida),
        }),
        Ok(RespostaDoEnvio::Redirecionamento(_)) => Ok(Some(FalhaDoSalvar::Redirecionamento302)),
        Err(erro) if enviado => Ok(Some(FalhaDoSalvar::Envio(erro))),
        Err(erro) => Err(D::Leitura(E::Submissao(erro))),
    }
}

/// Tabela de reconciliação do contrato.
fn conciliar(
    unidade: String,
    plano: PlanoDasColunas,
    campos_do_envio: Vec<String>,
    falha: Option<FalhaDoSalvar>,
    verificacao: VerificacaoDasColunas,
) -> Result<ColunasDefinidas, D> {
    let observadas: Vec<ColunaDoDetalhe> = match &verificacao {
        VerificacaoDasColunas::Marcadas(marcadas) => marcadas.clone(),
        VerificacaoDasColunas::LeituraFalhou(_) => {
            return Err(D::NaoConfirmadoNaConfiguracaoDeColunas {
                plano: Box::new(plano),
                falha_do_salvar: falha,
                verificacao: Box::new(verificacao),
            });
        }
    };
    if observadas == plano.alvo {
        return Ok(ColunasDefinidas {
            unidade,
            plano,
            estado: EstadoDasColunas::Conferidas {
                campos_do_envio,
                depois: observadas,
                reconciliado_por: falha,
            },
        });
    }
    let plano = Box::new(plano);
    if observadas == plano.antes {
        return Err(D::ConfiguracaoDeColunasNaoAplicada {
            plano,
            falha_do_salvar: falha,
        });
    }
    match falha {
        None => Err(D::MarcasDiferentesDoAlvoAposSalvar { plano, observadas }),
        Some(falha) => Err(D::NaoConfirmadoNaConfiguracaoDeColunas {
            plano,
            falha_do_salvar: Some(falha),
            verificacao: Box::new(verificacao),
        }),
    }
}
