//! Marcadores de `ies processo NUP` (contracts/cli/processo-obter-marcadores.md):
//! o link `andamento_marcador_gerenciar` da barra do nó do processo, lido
//! por [`crate::arvore_processo::ler_link_dos_marcadores`], e a tabela
//! `table#tblMarcadores` da tela "Marcadores do Processo", lida por GET.
//! A tela também tem escrita (`btnAdicionar`, `btnRemover`,
//! `frmGerenciarMarcador` com `andamento_marcador_remover`): nada disso é
//! enviado, clicado ou seguido. Um processo pode ter **vários** marcadores:
//! toda saída é lista. Estrutura em
//! docs/como-o-sei-funciona/marcadores-do-processo.md.
mod tela;

#[cfg(test)]
pub(crate) mod tests_apoio;
#[cfg(test)]
mod tests_tela;

pub(crate) use tela::ler_marcadores;

/// `<id>#<nome>`: o nome para a semântica, o id (só dígitos, antes do
/// primeiro `#`) para referência sem ambiguidade. O nome pode conter `#`.
pub(crate) fn rotulo_do_marcador(id: &str, nome: &str) -> String {
    format!("{id}#{nome}")
}

/// URL absoluta e assinada da tela: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDosMarcadores {
    url: String,
}

impl EnderecoDosMarcadores {
    pub(crate) fn novo(url: String) -> Self {
        Self { url }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.url
    }
}

impl std::fmt::Debug for EnderecoDosMarcadores {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDosMarcadores(<redigido>)")
    }
}

/// Uma linha de `tblMarcadores`; `None` é "não observado" (célula vazia).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MarcadorDoProcesso {
    /// `value` do checkbox da linha (o mesmo `ID-<n>` da âncora e do
    /// `tdAndamentoMarcador<n>`); opaco.
    pub(crate) id: String,
    pub(crate) nome: String,
    /// De `svg/marcador_<cor>.svg`.
    pub(crate) cor: String,
    pub(crate) texto: Option<String>,
    /// Sigla de `a.ancoraSigla`.
    pub(crate) usuario: Option<String>,
    /// `dd/mm/aaaa hh:mm`.
    pub(crate) data_hora: String,
}

/// A tabela lida: a legenda e as linhas, na ordem servida.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MarcadoresLidos {
    pub(crate) total_declarado: usize,
    pub(crate) itens: Vec<MarcadorDoProcesso>,
}

/// O que a tela, com a identidade conferida, mostrou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TelaDosMarcadores {
    Lidos(MarcadoresLidos),
    /// A tela é a deste processo, mas sem `table#tblMarcadores` (forma de
    /// zero marcadores é o 302 para `andamento_marcador_cadastrar`, tratado
    /// pelo fluxo; esta forma não foi observada e não vira `[]`).
    SemTabela,
}

/// Falhas do link na barra e da tela: um erro por fato.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroMarcadores {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[from] crate::html_atributos::ErroAtributoHtml),
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    LinksDosMarcadoresRepetidosNaBarra,
    EnderecoDosMarcadoresNaoReconhecido,
    EnderecoDosMarcadoresDeOutroProcesso,
    TelaDosMarcadoresAusente,
    /// A `action` do formulário de marcadores não se deixa ler em pares.
    ConsultaDoFormularioIlegivel(
        #[source] crate::consulta_do_controlador::ErroConsultaDoControlador,
    ),
    /// A `acao` do formulário de marcadores não é única nem conhecida.
    AcaoDoFormularioNaoLida(#[source] crate::consulta_do_controlador::ErroChaveDaConsulta),
    /// O `id_procedimento` do formulário não é único.
    IdDoFormularioNaoLido(#[source] crate::consulta_do_controlador::ErroChaveDaConsulta),
    NumeroDosMarcadoresDiferenteDoSolicitado,
    IdDosMarcadoresDiferenteDoAberto,
    MarcadoresAlemDoTotalDeclarado,
    CabecalhoDosMarcadoresNaoReconhecido,
    HtmlDosMarcadoresNaoInterpretado,
    /// O parser de HTML recusou a tela de marcadores.
    HtmlDosMarcadoresRecusadoPeloParser(#[source] tl::ParseError),
    /// O número da legenda dos marcadores não é um inteiro decimal.
    NumeroDaLegendaIlegivel(#[source] std::num::ParseIntError),
    LegendaDosMarcadoresForaDaFormaObservada,
    LinhaDoMarcadorForaDaFormaObservada,
    MarcadorRepetidoNaTela,
    TabelasDosMarcadoresRepetidas,
}

impl ErroMarcadores {
    /// A página não é a tela deste processo: um link guardado errou.
    pub(crate) fn link_envelhecido(&self) -> bool {
        matches!(
            self,
            Self::TelaDosMarcadoresAusente
                | Self::NumeroDosMarcadoresDiferenteDoSolicitado
                | Self::IdDosMarcadoresDiferenteDoAberto
        )
    }
}
