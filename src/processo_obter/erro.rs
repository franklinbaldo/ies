//! Falhas de `ies processo NUP`: um erro por fato
//! verificável (contracts/cli/processo-obter-arvore.md).
use crate::arvore_processo::{ErroArvore, ErroTelaDoProcesso};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::historico_processo::{ErroAndamentos, ErroLinkDoHistorico};
use crate::marcadores_processo::ErroMarcadores;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::redirecionamento::ErroRedirecionamentoDaPesquisa;
use crate::propriedades_processo::ErroPropriedades;
use crate::transporte::FalhaTransporte;

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
    /// A entrada por id respondeu HTML em vez do 302 observado.
    RespostaDaEntradaPorIdNaoReconhecida,
    /// A pesquisa rápida respondeu HTML em vez do 302 para `procedimento_trabalhar`.
    PesquisaRapidaSemUrlDeTrabalho,
    /// 302 fora das formas observadas (nem trabalho, nem login), ou HTML
    /// onde se esperava redirecionamento.
    RedirecionamentoNaoReconhecido,
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
    /// A barra não oferece `procedimento_alterar` nem `procedimento_consultar`.
    PropriedadesNaoPublicadasNaBarra,
    /// A barra não oferece `andamento_marcador_gerenciar`.
    MarcadoresNaoPublicadosNaBarra,
    /// A tela de marcadores é a deste processo, mas sem `table#tblMarcadores`.
    TabelaDeMarcadoresNaoPublicadaNaTela,
    LinksDoProcessoRepetidosNoControle,
    ProcessoRepetidoNoControle,
}

impl ErroProcessoObter {
    /// A sessão do SEI acabou (decide o relogin único): por `match` no tipo, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Transporte(erro) => erro.sessao_encerrada(),
            Self::Controle(erro) => erro.sessao_encerrada(),
            Self::SessaoEncerrada(_) => true,
            Self::Formulario(erro) => erro.sessao_encerrada(),
            Self::ValidacaoDeAcesso(erro) => erro.sessao_encerrada(),
            Self::FormularioDoHistorico(erro) => erro.sessao_encerrada(),
            Self::EntradaPorIdRecusada
            | Self::RespostaDaEntradaPorIdNaoReconhecida
            | Self::PesquisaRapidaSemUrlDeTrabalho
            | Self::RedirecionamentoNaoReconhecido
            | Self::Redirecionamento(_)
            | Self::LinkDoProcessoNoControleNaoReconhecido(_)
            | Self::CharsetDoControleNaoSuportado
            | Self::Tela(_)
            | Self::Arvore(_)
            | Self::LinkDoHistorico(_)
            | Self::Andamentos(_)
            | Self::Propriedades(_)
            | Self::Marcadores(_)
            | Self::PropriedadesNaoPublicadasNaBarra
            | Self::MarcadoresNaoPublicadosNaBarra
            | Self::TabelaDeMarcadoresNaoPublicadaNaTela
            | Self::LinksDoProcessoRepetidosNoControle
            | Self::ProcessoRepetidoNoControle => false,
        }
    }
}
