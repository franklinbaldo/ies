//! Falhas de `ies processo NUP`: uma variante por fato
//! verificável, com subcódigo estável (contracts/cli/processo-obter-arvore.md).
use crate::arvore_processo::{ErroArvore, ErroTelaDoProcesso};
use crate::erro_html::ErroHtml;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::historico_processo::{ErroAndamentos, ErroLinkDoHistorico};
use crate::marcadores_processo::ErroMarcadores;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::redirecionamento::ErroRedirecionamentoDaPesquisa;
use crate::propriedades_processo::ErroPropriedades;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroProcessoObter {
    /// Cookie, URL privada, rede ou status de uma requisição.
    Transporte(#[from] FalhaTransporte),
    /// GET do Controle (sessão, reconhecimento, unidade): os de `processo buscar`.
    Controle(#[from] ErroBuscaDeProcesso),
    /// A entrada por id respondeu a recusa característica ("Link sem
    /// assinatura" no `login.php`); a sessão morreu.
    EntradaPorIdRecusada,
    /// Tela de login (200 ou 302 sem a mensagem de recusa) fora de palpite.
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    /// A entrada por id respondeu outra coisa que o 302 observado.
    RespostaDaEntradaPorIdNaoReconhecida(StatusHttpRecusado),
    /// A pesquisa rápida não respondeu o 302 para `procedimento_trabalhar`.
    PesquisaRapidaSemUrlDeTrabalho(StatusHttpRecusado),
    /// 302 fora das formas observadas (nem trabalho, nem login).
    RedirecionamentoNaoReconhecido(StatusHttpRecusado),
    /// O 302 para `procedimento_trabalhar` não passou na conferência.
    Redirecionamento(#[from] ErroRedirecionamentoDaPesquisa),
    LinkDoProcessoNoControleNaoReconhecido(ErroRedirecionamentoDaPesquisa),
    CharsetDoControleNaoSuportado,
    /// Derivação ou envio do formulário da pesquisa rápida.
    Formulario(#[from] ErroSubmissaoFormulario),
    ValidacaoDeAcesso(#[from] super::validacao_acesso::ErroValidacaoAcesso),
    Tela(#[from] ErroTelaDoProcesso),
    Arvore(#[from] ErroArvore),
    /// O link do histórico na árvore (só quando os andamentos são lidos).
    LinkDoHistorico(#[from] ErroLinkDoHistorico),
    /// A página do histórico ou a troca de visão.
    Andamentos(#[from] ErroAndamentos),
    /// Derivação ou envio do formulário `verHistorico('P')`.
    FormularioDoHistorico(#[from] super::andamentos::HistoricoNaoPedido),
    /// O link das propriedades na barra ou a tela delas.
    Propriedades(#[from] ErroPropriedades),
    /// O link dos marcadores na barra ou a tela deles.
    Marcadores(#[from] ErroMarcadores),
    Html(#[from] ErroHtml),
}

impl ErroProcessoObter {
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Html(e) => e.mensagem,
            Self::Transporte(erro) => erro.mensagem(),
            Self::Controle(erro) => erro.mensagem(),
            Self::EntradaPorIdRecusada => {
                "a instalação recusou a entrada sem assinatura e encerrou a sessão; rode ies login"
            }
            Self::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
                "o SEI devolveu a tela de login; rode ies login"
            }
            Self::RespostaDaEntradaPorIdNaoReconhecida(_) => {
                "a entrada por id não respondeu como observado"
            }
            Self::PesquisaRapidaSemUrlDeTrabalho(_) => {
                "a pesquisa rápida não abriu o processo (resultado não é lido)"
            }
            Self::RedirecionamentoNaoReconhecido(_) => "o SEI redirecionou para fora do observado",
            Self::Redirecionamento(_) => "o endereço do processo não foi reconhecido",
            Self::LinkDoProcessoNoControleNaoReconhecido(_) => {
                "o link do processo no Controle não foi reconhecido"
            }
            Self::CharsetDoControleNaoSuportado => "o charset do Controle não é suportado",
            Self::Formulario(_) => "a pesquisa rápida não foi enviada como observado",
            Self::ValidacaoDeAcesso(super::validacao_acesso::ErroValidacaoAcesso::Credenciais(
                _,
            )) => "o arquivo de credenciais privado está ausente, inseguro ou inválido",
            Self::ValidacaoDeAcesso(erro) if erro.status_http().is_some() => {
                "o SEI recusou a validação de acesso; não repetir"
            }
            Self::ValidacaoDeAcesso(_) => {
                "a modal de validação de acesso não foi servida como esperado"
            }
            Self::Tela(ErroTelaDoProcesso::UnidadeDaTelaDiferenteDaSolicitada) => {
                "a unidade da tela do processo difere da solicitada"
            }
            Self::Tela(ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso) => {
                // O fato do SEI é tipado. A ausência da configuração privada
                // é um detalhe da mensagem, não um novo código de erro.
                if std::env::var_os("IES_CREDENCIAIS_FILE").is_none_or(|caminho| caminho.is_empty())
                {
                    "o SEI exige validar o acesso com a senha antes de abrir este processo; falta o arquivo de credenciais privado (IES_CREDENCIAIS_FILE); o IES ainda não faz essa validação"
                } else {
                    "o SEI exige validar o acesso com a senha antes de abrir este processo; o IES ainda não faz essa validação"
                }
            }
            Self::Tela(_) => "a tela do processo não foi reconhecida",
            Self::Arvore(ErroArvore::IdentidadeDoProcessoDiferenteDaPedida) => {
                "o processo aberto não tem o número pedido"
            }
            Self::Arvore(_) => "a árvore do processo não foi reconhecida",
            Self::LinkDoHistorico(_) => "o link do histórico na árvore não foi reconhecido",
            Self::Andamentos(ErroAndamentos::Historico(_)) => {
                "a página do histórico não foi reconhecida"
            }
            Self::Andamentos(ErroAndamentos::AndamentosAlemDoTotalDeclarado) => {
                "o histórico mostrou mais andamentos que o total declarado"
            }
            Self::Andamentos(ErroAndamentos::VisaoDoHistoricoDiferenteDaPedida) => {
                "o SEI não serviu a visão completa do histórico"
            }
            Self::Andamentos(_) => "o histórico não foi servido na forma observada",
            Self::FormularioDoHistorico(_) => "o histórico completo não foi pedido como observado",
            Self::Propriedades(erro) if erro.do_link() => {
                "o link das propriedades na barra do processo não foi reconhecido"
            }
            Self::Propriedades(
                ErroPropriedades::NumeroDasPropriedadesDiferenteDoSolicitado
                | ErroPropriedades::IdDasPropriedadesDiferenteDoAberto,
            ) => "a tela de propriedades é de outro processo",
            Self::Propriedades(_) => "a tela de propriedades não foi servida na forma observada",
            Self::Marcadores(erro) if erro.do_link() => {
                "o link dos marcadores na barra do processo não foi reconhecido"
            }
            Self::Marcadores(
                ErroMarcadores::NumeroDosMarcadoresDiferenteDoSolicitado
                | ErroMarcadores::IdDosMarcadoresDiferenteDoAberto,
            ) => "a tela de marcadores é de outro processo",
            Self::Marcadores(ErroMarcadores::MarcadoresAlemDoTotalDeclarado) => {
                "a tela de marcadores mostrou mais linhas que o total declarado"
            }
            Self::Marcadores(_) => "a tela de marcadores não foi servida na forma observada",
        }
    }

    /// Status e mensagem do SEI, da requisição recusada.
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(erro) => erro.status_http(),
            Self::Controle(erro) => erro.status_http(),
            Self::RespostaDaEntradaPorIdNaoReconhecida(status)
            | Self::PesquisaRapidaSemUrlDeTrabalho(status)
            | Self::RedirecionamentoNaoReconhecido(status) => Some(status),
            Self::Formulario(erro) => erro.status_http(),
            Self::ValidacaoDeAcesso(erro) => erro.status_http(),
            Self::FormularioDoHistorico(erro) => erro.status_http(),
            _ => None,
        }
    }
}
