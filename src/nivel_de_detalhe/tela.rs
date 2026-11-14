//! Leitura da tela `procedimento_configurar_detalhe` (GET, somente leitura).
//! Nada é salvo: `sbmSalvar` não é enviado por este módulo.
use super::{ColunaDoDetalhe, NivelDeDetalhe, OpcaoDoDetalhe};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{ConsultaDoControlador, ErroConsultaDoControlador};
use crate::formulario_web::{ErroSubmissaoFormulario, localizar_formulario};
use std::collections::HashMap;

/// Formulário observado (o `id` é o publicado pelo SEI, apesar do nome).
pub(crate) const FORMULARIO_DO_NIVEL_DE_DETALHE: &str = "frmAcessoExternoCancelar";

/// um erro por fato verificável da tela de configuração.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroNivelDeDetalhe {
    AcaoDoFormularioDoNivelDeDetalheDiferente,
    /// A `action` do formulário não se deixa ler em pares `chave=valor`.
    ConsultaDoFormularioIlegivel(#[source] ErroConsultaDoControlador),
    /// A `acao` do formulário não é única nem conhecida.
    AcaoDoFormularioNaoLida(#[source] crate::consulta_do_controlador::ErroAcaoDaUrl),
    /// O formulário não publica nenhuma das 12 caixas do catálogo.
    /// Uma caixa do catálogo ausente não é erro: sai com `disponivel: false`.
    NenhumaOpcaoDoDetalhePublicada,
    OpcaoDoDetalheRepetida {
        coluna: ColunaDoDetalhe,
    },
    RotuloDaOpcaoDoDetalheDiferente {
        coluna: ColunaDoDetalhe,
    },
    CampoNaoPrevistoNoFormularioDoNivelDeDetalhe,
    FormularioDoNivelDeDetalheAusente(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    FormulariosDoNivelDeDetalheRepetidos,
    OpcaoDoNivelDeDetalheNaoPrevista,
}

type E = ErroNivelDeDetalhe;

/// Texto visível com espaços (inclusive `&nbsp;`) colapsados.
pub(crate) fn texto_colapsado(texto: &str) -> String {
    texto
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `action` com exatamente um `acao=procedimento_configurar_detalhe`.
fn conferir_acao(form: crate::tela_html::Elemento<'_>) -> Result<(), E> {
    let diferente = || E::AcaoDoFormularioDoNivelDeDetalheDiferente;
    let acao = form
        .atributo("action")
        .map(str::to_owned)
        .ok_or_else(diferente)?;
    let (_, consulta) = acao.split_once('?').ok_or_else(diferente)?;
    let pares = ConsultaDoControlador::ler(consulta).map_err(E::ConsultaDoFormularioIlegivel)?;
    let lida = pares.acao().map_err(E::AcaoDoFormularioNaoLida)?;
    if lida == AcaoDoSei::ProcedimentoConfigurarDetalhe.como_na_url() {
        Ok(())
    } else {
        Err(diferente())
    }
}

/// As 12 opções do catálogo: as que a tela publica (`disponivel`) e as
/// marcadas. A tela pode publicar um subconjunto (`UnB` 4.0.9: 11, sem
/// Prioridade); caixa `chkSin*` fora do catálogo é recusa.
pub(crate) fn ler_nivel_de_detalhe(
    html: &crate::tela_html::HtmlServido,
) -> Result<NivelDeDetalhe, ErroNivelDeDetalhe> {
    let dom = crate::tela_html::arvore(html);
    let form =
        localizar_formulario(&dom, FORMULARIO_DO_NIVEL_DE_DETALHE).map_err(|erro| match erro {
            ErroSubmissaoFormulario::FormulariosComMesmoIdentificadorNaTela => {
                E::FormulariosDoNivelDeDetalheRepetidos
            }
            outro @ (ErroSubmissaoFormulario::EfeitoDaOperacaoNaoAvaliado
            | ErroSubmissaoFormulario::UnidadeDaTelaDiferenteDaSolicitada
            | ErroSubmissaoFormulario::UnidadeNaoInformada
            | ErroSubmissaoFormulario::MetodoDoFormularioDiferenteDePost
            | ErroSubmissaoFormulario::EnctypeDoFormularioNaoUrlencoded
            | ErroSubmissaoFormulario::AcceptCharsetPublicadoNoFormulario
            | ErroSubmissaoFormulario::AcaoDoFormularioNaoEhUrlHttps
            | ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao
            | ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao
            | ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico
            | ErroSubmissaoFormulario::CampoOcultoAusenteNoFormulario { .. }
            | ErroSubmissaoFormulario::CampoOcultoRepetidoNoFormulario { .. }
            | ErroSubmissaoFormulario::HdnAcaoPublicadoSemRegraDaOperacao
            | ErroSubmissaoFormulario::CampoForaDaListaDaOperacao { .. }
            | ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario { .. }
            | ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez { .. }
            | ErroSubmissaoFormulario::CampoCriadoPeloScriptJaPublicado { .. }
            | ErroSubmissaoFormulario::ValorNaoRepresentavelNaCodificacao { .. }
            | ErroSubmissaoFormulario::TelaDeLoginRecebidaAposEnvio
            | ErroSubmissaoFormulario::MarcadorDeResultadoAusenteNaResposta
            | ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(_)
            | ErroSubmissaoFormulario::Transporte(_)
            | ErroSubmissaoFormulario::AcaoDoFormularioAusente
            | ErroSubmissaoFormulario::BotaoDeEnvioAusenteNoFormulario
            | ErroSubmissaoFormulario::BotoesDeEnvioRepetidosNoFormulario
            | ErroSubmissaoFormulario::FormularioAusenteNaTela
            | ErroSubmissaoFormulario::IdentificacaoDaUnidadeAusenteNaTela
            | ErroSubmissaoFormulario::IdentificacoesDaUnidadeRepetidasNaTela(_)
            | ErroSubmissaoFormulario::ConsultaDaAcaoIlegivel(_)) => {
                E::FormularioDoNivelDeDetalheAusente(outro)
            }
        })?;
    conferir_acao(form)?;
    let mut marcadas: HashMap<ColunaDoDetalhe, bool> = HashMap::new();
    let mut rotulos: HashMap<String, String> = HashMap::new();
    for tag in form.descendentes() {
        let elemento = tag.nome().to_ascii_lowercase();
        if elemento == "label" {
            if let Some(alvo) = tag.atributo("for").map(str::to_owned) {
                rotulos.insert(alvo, texto_colapsado(&tag.texto()));
            }
            continue;
        }
        let caixa = elemento == "input"
            && tag
                .atributo("type")
                .is_some_and(|t| t.eq_ignore_ascii_case("checkbox"));
        let Some(nome) = tag
            .atributo("name")
            .filter(|n| caixa && n.starts_with("chkSin"))
            .map(str::to_owned)
        else {
            continue;
        };
        let coluna = ColunaDoDetalhe::pelo_campo(&nome)
            .ok_or(ErroNivelDeDetalhe::OpcaoDoNivelDeDetalheNaoPrevista)?;
        let marcada = tag.presente("checked");
        if marcadas.insert(coluna, marcada).is_some() {
            return Err(E::OpcaoDoDetalheRepetida { coluna });
        }
    }
    if marcadas.is_empty() {
        return Err(E::NenhumaOpcaoDoDetalhePublicada);
    }
    let mut opcoes = Vec::with_capacity(ColunaDoDetalhe::TODAS.len());
    for coluna in ColunaDoDetalhe::TODAS {
        let Some(&marcada) = marcadas.get(&coluna) else {
            opcoes.push(OpcaoDoDetalhe {
                coluna,
                disponivel: false,
                marcada: false,
            });
            continue;
        };
        if rotulos.get(coluna.campo()).map(String::as_str) != Some(coluna.rotulo()) {
            return Err(E::RotuloDaOpcaoDoDetalheDiferente { coluna });
        }
        opcoes.push(OpcaoDoDetalhe {
            coluna,
            disponivel: true,
            marcada,
        });
    }
    Ok(NivelDeDetalhe { opcoes })
}
