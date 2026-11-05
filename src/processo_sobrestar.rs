//! Prévia Web dos comandos sobrestar/retomar (#269).
//! Confirmação continua fechada antes da rede nesta fatia: não existe POST
//! seguro sem conferir formulário e reconciliação completa de escrita.
use crate::arvore_processo::{AbertoEm, ErroAcaoSobrestar, MotivoAbertoEm};
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::leitura;
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::StatusHttpRecusado;

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
    ConfirmacaoAindaBloqueada,
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
            Self::ConfirmacaoAindaBloqueada => "confirmacao_de_sobrestamento_nao_implementada",
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
            Self::ConfirmacaoAindaBloqueada => {
                "a confirmação está bloqueada até implementar e testar a escrita única e sua verificação"
            }
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Abertura(e) => e.status_http(),
            _ => None,
        }
    }
}

/// Prévia: só a abertura do processo, a barra e a situação conferida.
/// A ação não publica nem executa a URL assinada.
pub(crate) fn preparar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade: Option<&str>,
    acao: Acao,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, ErroSobrestamento> {
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
    if oferecida.is_none() {
        return Err(ErroSobrestamento::AcaoNaoOferecida);
    }
    Ok(Previa {
        nup: nup.as_str().to_owned(),
        id_procedimento: aberto.arvore.processo.id,
        unidade: atual,
        acao,
        requisicoes: aberto.requisicoes,
    })
}

#[cfg(test)]
mod tests;
