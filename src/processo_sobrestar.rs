//! Prévia e confirmação única de sobrestar/retomar (#269).
//! A confirmação deriva a escrita da tela e exige leitura verificadora.
use crate::arvore_processo::{
    AbertoEm, EnderecoDeEscrita, ErroAcaoSobrestar, ErroEnderecoDeRetomar, MotivoAbertoEm,
};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::transporte::FalhaTransporte;
mod fluxo;
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::leitura;
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
pub(crate) use fluxo::executar;

/// `--motivo` vazio ou só com espaços.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct MotivoDoSobrestamentoVazio;

/// `--motivo`: o clap recusa o vazio (saída 2) antes da sessão.
pub(crate) fn motivo_do_sobrestamento(motivo: &str) -> Result<String, MotivoDoSobrestamentoVazio> {
    if motivo.trim().is_empty() {
        Err(MotivoDoSobrestamentoVazio)
    } else {
        Ok(motivo.to_owned())
    }
}

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

/// Um pedido de escrita, sem URL nem cookie. Os parâmetros permanecem
/// associados para a prévia e para a confirmação da mesma ação.
#[derive(Clone, Copy)]
pub(crate) struct PedidoDaMudanca<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) acao: Acao,
    pub(crate) motivo: Option<&'a str>,
    pub(crate) confirmar: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Previa {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: String,
    pub(crate) acao: Acao,
    pub(crate) requisicoes: u8,
    pub(crate) estado: EstadoDaMudanca,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDaMudanca {
    Previa,
    Aplicada,
}

impl EstadoDaMudanca {
    pub(crate) fn aplicada(self) -> bool {
        matches!(self, Self::Aplicada)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroSobrestamento {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(#[from] CookieIlegivel),
    Abertura(#[from] ErroProcessoObter),
    UnidadeNaoPublicada,
    SituacaoNaoLida(MotivoAbertoEm),
    Barra(#[from] ErroAcaoSobrestar),
    JaSobrestado,
    NaoSobrestado,
    ForaDaUnidade,
    SobrestadoEmOutraUnidade,
    AcaoNaoOferecida,
    MotivoVazio,
    TelaDaEscrita(#[from] FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    CharsetNaoSuportado,
    TelaSobrestar(#[from] tela::ErroTela),
    Formulario(#[from] ErroSubmissaoFormulario),
    EnderecoDeRetomar(#[from] ErroEnderecoDeRetomar),
    EscritaNaoAplicada,
    NaoConfirmado,
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
    let (aberto, acoes) = abrir_para_escrever(sessao, &pedido, indice, envio)?;
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
    }?;
    let endereco = oferecida.ok_or(ErroSobrestamento::AcaoNaoOferecida)?;
    Ok((
        Previa {
            nup: nup.as_str().to_owned(),
            id_procedimento: aberto.arvore.processo.id,
            unidade: atual,
            acao,
            requisicoes: aberto.requisicoes,
            estado: EstadoDaMudanca::Previa,
        },
        endereco,
    ))
}

#[cfg(test)]
mod acervo;
mod tela;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_escrita;
#[cfg(test)]
mod tests_tela;
