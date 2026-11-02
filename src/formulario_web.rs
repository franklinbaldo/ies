//! Submissão derivada de formulário HTML servido pelo SEI, sem rede.
//! Contrato: docs/arquitetura/0002-submeter-formulario-observado.md.
//! `action` aqui é o atributo do `<form>`, não um despachante genérico.
use crate::instalacao::Instalacao;
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::fmt;
use std::io::Read;
use std::path::Path;

/// Estado de sessão que uma operação pode alterar, mesmo quando consulta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDeSessao {
    PaginaOuFiltroLembrado,
    UnidadeAtual,
    PreferenciaDeVisualizacao,
}

/// Avaliação obrigatória por operação, sem valor padrão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EfeitoAvaliado {
    ConsultaSemEfeitoObservado { evidencia: &'static str },
    AlteraEstadoDaSessao { o_que: EstadoDeSessao },
    Desconhecido,
}

/// Declaração `const` feita pela própria operação; não há catálogo central.
#[derive(Debug)]
pub(crate) struct OperacaoDeFormulario {
    pub(crate) acao: &'static str,
    pub(crate) formulario: &'static str,
    pub(crate) botao_de_envio: &'static str,
    pub(crate) campos_ocultos_exigidos: &'static [&'static str],
    pub(crate) campos_usuario: &'static [&'static str],
    pub(crate) marcador_de_resultado: &'static str,
    pub(crate) efeito: EfeitoAvaliado,
}

/// Charset declarado pela resposta que serviu a tela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CodificacaoDaTela {
    Iso88591,
    Utf8,
}

/// HTML e URL exata da mesma resposta, na mesma geração de sessão.
pub(crate) struct TelaServida<'a> {
    pub(crate) url: &'a str,
    pub(crate) html: &'a str,
    pub(crate) codificacao: CodificacaoDaTela,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroSubmissaoFormulario {
    EfeitoDaOperacaoNaoAvaliado,
    IdentificacaoDaUnidadeAusenteNaTela,
    IdentificacoesDaUnidadeRepetidasNaTela,
    UnidadeDaTelaDiferenteDaSolicitada,
    FormularioAusenteNaTela,
    FormulariosComMesmoIdentificadorNaTela,
    MetodoDoFormularioDiferenteDePost,
    EnctypeDoFormularioNaoUrlencoded,
    AcceptCharsetPublicadoNoFormulario,
    AcaoDoFormularioAusente,
    AcaoDoFormularioNaoEhUrlHttps,
    AcaoDoFormularioForaDaInstalacao,
    AcaoDoFormularioDiferenteDaOperacao,
    AcaoDoFormularioSemInfraHashUnico,
    EntidadeHtmlDesconhecidaNoAtributo { campo: String },
    CampoOcultoAusenteNoFormulario { campo: String },
    CampoOcultoRepetidoNoFormulario { campo: String },
    HdnAcaoPublicadoSemRegraDaOperacao,
    CampoForaDaListaDaOperacao { campo: String },
    CampoNaoPublicadoPeloFormulario { campo: String },
    CampoInformadoMaisDeUmaVez { campo: String },
    ValorNaoRepresentavelNaCodificacao { campo: String },
    BotaoDeEnvioAusenteNoFormulario,
    BotoesDeEnvioRepetidosNoFormulario,
    TelaDeLoginRecebidaAposEnvio,
    MarcadorDeResultadoAusenteNaResposta,
    Transporte(FalhaTransporte),
}

impl ErroSubmissaoFormulario {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::EfeitoDaOperacaoNaoAvaliado => "efeito_da_operacao_nao_avaliado",
            Self::IdentificacaoDaUnidadeAusenteNaTela => "identificacao_da_unidade_ausente_na_tela",
            Self::IdentificacoesDaUnidadeRepetidasNaTela => {
                "identificacoes_da_unidade_repetidas_na_tela"
            }
            Self::UnidadeDaTelaDiferenteDaSolicitada => "unidade_da_tela_diferente_da_solicitada",
            Self::FormularioAusenteNaTela => "formulario_ausente_na_tela",
            Self::FormulariosComMesmoIdentificadorNaTela => {
                "formularios_com_mesmo_identificador_na_tela"
            }
            Self::MetodoDoFormularioDiferenteDePost => "metodo_do_formulario_diferente_de_post",
            Self::EnctypeDoFormularioNaoUrlencoded => "enctype_do_formulario_nao_urlencoded",
            Self::AcceptCharsetPublicadoNoFormulario => "accept_charset_publicado_no_formulario",
            Self::AcaoDoFormularioAusente => "acao_do_formulario_ausente",
            Self::AcaoDoFormularioNaoEhUrlHttps => "acao_do_formulario_nao_eh_url_https",
            Self::AcaoDoFormularioForaDaInstalacao => "acao_do_formulario_fora_da_instalacao",
            Self::AcaoDoFormularioDiferenteDaOperacao => "acao_do_formulario_diferente_da_operacao",
            Self::AcaoDoFormularioSemInfraHashUnico => "acao_do_formulario_sem_infra_hash_unico",
            Self::EntidadeHtmlDesconhecidaNoAtributo { .. } => {
                "entidade_html_desconhecida_no_atributo"
            }
            Self::CampoOcultoAusenteNoFormulario { .. } => "campo_oculto_ausente_no_formulario",
            Self::CampoOcultoRepetidoNoFormulario { .. } => "campo_oculto_repetido_no_formulario",
            Self::HdnAcaoPublicadoSemRegraDaOperacao => "hdn_acao_publicado_sem_regra_da_operacao",
            Self::CampoForaDaListaDaOperacao { .. } => "campo_fora_da_lista_da_operacao",
            Self::CampoNaoPublicadoPeloFormulario { .. } => "campo_nao_publicado_pelo_formulario",
            Self::CampoInformadoMaisDeUmaVez { .. } => "campo_informado_mais_de_uma_vez",
            Self::ValorNaoRepresentavelNaCodificacao { .. } => {
                "valor_nao_representavel_na_codificacao"
            }
            Self::BotaoDeEnvioAusenteNoFormulario => "botao_de_envio_ausente_no_formulario",
            Self::BotoesDeEnvioRepetidosNoFormulario => "botoes_de_envio_repetidos_no_formulario",
            Self::TelaDeLoginRecebidaAposEnvio => "tela_de_login_recebida_apos_envio",
            Self::MarcadorDeResultadoAusenteNaResposta => {
                "marcador_de_resultado_ausente_na_resposta"
            }
            Self::Transporte(falha) => falha.subcodigo(),
        }
    }
}

/// POST derivado; não é `Clone`, e o envio o consome (sem retry).
pub(crate) struct SubmissaoDerivada {
    url: String,
    host: String,
    referer: String,
    nomes: Vec<String>,
    corpo: Vec<u8>,
    efeito: EfeitoAvaliado,
    marcador_de_resultado: &'static str,
}

impl SubmissaoDerivada {
    pub(crate) fn metodo(&self) -> &'static str {
        "POST"
    }
    pub(crate) fn url(&self) -> &str {
        &self.url
    }
    pub(crate) fn origem(&self) -> String {
        format!("https://{}", self.host)
    }
    pub(crate) fn referer(&self) -> &str {
        &self.referer
    }
    pub(crate) fn tipo_de_conteudo(&self) -> &'static str {
        "application/x-www-form-urlencoded"
    }
    pub(crate) fn nomes_dos_campos(&self) -> &[String] {
        &self.nomes
    }
    pub(crate) fn corpo(&self) -> &[u8] {
        &self.corpo
    }
    pub(crate) fn efeito(&self) -> EfeitoAvaliado {
        self.efeito
    }
}

impl fmt::Debug for SubmissaoDerivada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = (
            &self.url,
            &self.referer,
            &self.corpo,
            self.marcador_de_resultado,
        );
        unimplemented!("CONTRACT shim: redação do Debug")
    }
}

/// Deriva a submissão do formulário servido, sem rede.
pub(crate) fn derivar_submissao(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    let _ = (operacao, instalacao, tela, unidade_solicitada, campos);
    unimplemented!("CONTRACT shim: derivação")
}

/// Envio injetável: `FnOnce` recebe a submissão e o cookie, uma vez.
pub(crate) fn enviar_submissao_via<R: Read, F>(
    submissao: SubmissaoDerivada,
    cookie_file: &Path,
    enviar: F,
) -> Result<String, ErroSubmissaoFormulario>
where
    F: FnOnce(&SubmissaoDerivada, &str) -> Result<RespostaWeb<R>, FalhaTransporte>,
{
    let _ = (submissao, cookie_file, enviar);
    unimplemented!("CONTRACT shim: envio")
}

#[cfg(test)]
#[path = "formulario_web/tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "formulario_web/tests_envio.rs"]
mod tests_envio;
