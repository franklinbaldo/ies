//! Prévia e confirmação única de sobrestar/retomar (#269).
//! A confirmação deriva a escrita da tela e exige leitura verificadora.
use crate::arvore_processo::{
    AbertoEm, EnderecoDeEscrita, ErroAcaoSobrestar, ErroEnderecoDeRetomar, MotivoAbertoEm,
};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::transporte::FalhaTransporte;
mod fluxo;
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::leitura;
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::StatusHttpRecusado;
pub(crate) use fluxo::executar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Acao {
    Sobrestar,
    Retomar,
}

impl Acao {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::Sobrestar => "sobrestar",
            Self::Retomar => "retomar",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Previa {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: String,
    pub(crate) acao: Acao,
    pub(crate) requisicoes: u8,
    pub(crate) confirmado: bool,
    pub(crate) alterado: bool,
    pub(crate) reconciliado_por_leitura: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroSobrestamento {
    Abertura(ErroProcessoObter),
    UnidadeNaoPublicada,
    SituacaoNaoLida(MotivoAbertoEm),
    Barra(ErroAcaoSobrestar),
    JaSobrestado,
    NaoSobrestado,
    ForaDaUnidade,
    SobrestadoEmOutraUnidade,
    AcaoNaoOferecida,
    MotivoVazio,
    TelaDaEscrita(FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada,
    CharsetNaoSuportado,
    TelaSobrestar(tela::ErroTela),
    Formulario(ErroSubmissaoFormulario),
    EnderecoDeRetomar(ErroEnderecoDeRetomar),
    EscritaNaoAplicada,
    EfeitoIncerto,
}

impl ErroSobrestamento {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Abertura(e) => e.subcodigo(),
            Self::UnidadeNaoPublicada => "unidade_do_sobrestamento_nao_publicada",
            Self::SituacaoNaoLida(_) => "situacao_antes_do_sobrestamento_nao_lida",
            Self::Barra(e) => e.subcodigo(),
            Self::JaSobrestado => "processo_ja_sobrestado",
            Self::NaoSobrestado => "processo_nao_sobrestado",
            Self::ForaDaUnidade => "processo_nao_aberto_na_unidade_para_sobrestar",
            Self::SobrestadoEmOutraUnidade => "processo_sobrestado_em_outra_unidade",
            Self::AcaoNaoOferecida => "acao_de_sobrestamento_nao_oferecida",
            Self::MotivoVazio => "motivo_do_sobrestamento_vazio",
            Self::TelaDaEscrita(falha) => falha.subcodigo(),
            Self::TelaRedirecionada => "tela_de_sobrestamento_redirecionada",
            Self::SessaoEncerrada => "sessao_encerrada_rode_ies_login",
            Self::CharsetNaoSuportado => "charset_da_tela_de_sobrestar_nao_suportado",
            Self::TelaSobrestar(erro) => erro.subcodigo(),
            Self::Formulario(erro) => erro.subcodigo(),
            Self::EnderecoDeRetomar(erro) => erro.subcodigo(),
            Self::EscritaNaoAplicada => "sobrestamento_ou_retomada_nao_aplicados",
            Self::EfeitoIncerto => "efeito_incerto_no_sobrestamento",
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Abertura(e) => e.mensagem(),
            Self::UnidadeNaoPublicada => "o SEI não publicou a unidade da sessão",
            Self::SituacaoNaoLida(_) => "a situação do processo não foi observada com segurança",
            Self::Barra(_) => "a ação da barra não está na forma observada",
            Self::JaSobrestado => "o processo já está sobrestado; nenhuma escrita enviada",
            Self::NaoSobrestado => "o processo não está sobrestado; nenhuma escrita enviada",
            Self::ForaDaUnidade => "o processo não está aberto na unidade ativa",
            Self::SobrestadoEmOutraUnidade => "o sobrestamento está em outra unidade",
            Self::AcaoNaoOferecida => "a barra do SEI não oferece esta ação",
            Self::MotivoVazio => "motivo obrigatório para sobrestar",
            Self::TelaDaEscrita(_) | Self::TelaRedirecionada => {
                "a tela da escrita não foi servida; nenhuma escrita enviada"
            }
            Self::SessaoEncerrada => "sessão encerrada; rode ies login; nada enviado",
            Self::CharsetNaoSuportado => "charset da tela não suportado; nada enviado",
            Self::TelaSobrestar(_) => {
                "formulário de sobrestamento fora da forma observada; nada enviado"
            }
            Self::Formulario(_) => "formulário de sobrestamento não pôde ser enviado",
            Self::EnderecoDeRetomar(_) => {
                "endereço de retomar fora da forma observada; nada enviado"
            }
            Self::EscritaNaoAplicada => "releitura confirmou que a ação não foi aplicada",
            Self::EfeitoIncerto => {
                "efeito incerto após escrita; consulte processo antes de repetir"
            }
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Abertura(e) => e.status_http(),
            Self::TelaDaEscrita(e) => e.status_http(),
            Self::Formulario(e) => e.status_http(),
            _ => None,
        }
    }
}

/// Prévia sem escrita. A URL assinada só vive dentro do fluxo de confirmação.
#[cfg(test)]
pub(crate) fn preparar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade: Option<&str>,
    acao: Acao,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, ErroSobrestamento> {
    preparar_com_acao(sessao, nup, unidade, acao, indice, envio).map(|(previa, _)| previa)
}

pub(super) fn preparar_com_acao<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade: Option<&str>,
    acao: Acao,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<(Previa, EnderecoDeEscrita), ErroSobrestamento> {
    let pedido = leitura(nup, unidade, true);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &pedido, indice, envio).map_err(ErroSobrestamento::Abertura)?;
    let atual = aberto
        .unidade
        .ok_or(ErroSobrestamento::UnidadeNaoPublicada)?;
    match (&acao, &aberto.arvore.aberto_em) {
        (Acao::Sobrestar, AbertoEm::Sobrestado(_)) => {
            return Err(ErroSobrestamento::JaSobrestado);
        }
        (Acao::Sobrestar, AbertoEm::Unidades(unidades)) => {
            if !unidades.iter().any(|u| u.sigla == atual) {
                return Err(ErroSobrestamento::ForaDaUnidade);
            }
        }
        (Acao::Retomar, AbertoEm::Sobrestado(unidades)) => {
            if !unidades.iter().any(|u| u.sigla == atual) {
                return Err(ErroSobrestamento::SobrestadoEmOutraUnidade);
            }
        }
        (Acao::Retomar, AbertoEm::Unidades(_)) => {
            return Err(ErroSobrestamento::NaoSobrestado);
        }
        (_, AbertoEm::NaoLido(motivo)) => {
            return Err(ErroSobrestamento::SituacaoNaoLida(*motivo));
        }
    }
    let oferecida = match acao {
        Acao::Sobrestar => acoes.sobrestar,
        Acao::Retomar => acoes.retomar,
    }
    .map_err(ErroSobrestamento::Barra)?;
    let endereco = oferecida.ok_or(ErroSobrestamento::AcaoNaoOferecida)?;
    Ok((
        Previa {
            nup: nup.as_str().to_owned(),
            id_procedimento: aberto.arvore.processo.id,
            unidade: atual,
            acao,
            requisicoes: aberto.requisicoes,
            confirmado: false,
            alterado: false,
            reconciliado_por_leitura: false,
        },
        endereco,
    ))
}

mod tela;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_tela;
#[cfg(test)]
mod tests_escrita;
