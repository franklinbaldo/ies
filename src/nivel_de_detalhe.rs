//! Nível de detalhe da visão detalhada (`D`) do Controle de Processos:
//! o catálogo das 12 opções conhecidas da tela `procedimento_configurar_detalhe`
//! e a coluna que cada uma produz em `tblProcessosDetalhado`. A tela pode
//! publicar um subconjunto (`UnB` 4.0.9: 11, sem Prioridade; live 02/11/2026).
//! Contrato: contracts/cli/unidade-colunas.md.
//!
//! Evidência estrutural (capturas privadas de 02/11/2026, treinamento): a
//! tela de PEN1 (5.0.4) e Goiás (5.0.3) publica o formulário
//! `frmAcessoExternoCancelar` com 12 caixas `chkSin*` e `label for=`; na D,
//! nove opções viram cabeçalho de texto e três (prazo, devolver, aguardando)
//! viram cabeçalho só de ícone, rotulado por `aria-label` (5.0.x) ou pelo
//! argumento de `infraTooltipMostrar` (`UnB` 4.0.9).
mod alvo;
mod salvar;
mod tela;

#[cfg(test)]
pub(crate) use alvo::ErroDaListaDeColunas;
pub(crate) use alvo::{
    AlvoDasColunas, ColunaIndisponivelNestaInstalacao, PlanoDasColunas, planejar,
};
pub(crate) use salvar::{
    SALVAR_NIVEL_DE_DETALHE, SinalDoSalvar, campos_do_envio, conferir_controles_do_formulario,
    sinal_do_salvar,
};
pub(crate) use tela::{
    ACAO_DO_NIVEL_DE_DETALHE, ErroNivelDeDetalhe, ler_nivel_de_detalhe, texto_colapsado,
};

/// Como a coluna aparece no cabeçalho da tabela D.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CabecalhoDaColuna {
    /// Texto do cabeçalho, com espaços colapsados (`Última\xa0Mov.` vira `Última Mov.`).
    Texto(&'static str),
    /// Cabeçalho só com ícone; o rótulo vem do `aria-label` ou do tooltip.
    Icone(&'static str),
}

/// Uma das 12 opções fixas do Nível de detalhe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ColunaDoDetalhe {
    Atribuicao,
    Anotacao,
    TipoProcesso,
    Especificacao,
    Prioridade,
    Interessados,
    Observacao,
    ControlePrazo,
    RetornoDevolver,
    RetornoAguardando,
    UltimaMovimentacao,
    Marcadores,
}

impl ColunaDoDetalhe {
    /// Ordem da tela de configuração.
    pub(crate) const TODAS: [Self; 12] = [
        Self::Atribuicao,
        Self::Anotacao,
        Self::TipoProcesso,
        Self::Especificacao,
        Self::Prioridade,
        Self::Interessados,
        Self::Observacao,
        Self::ControlePrazo,
        Self::RetornoDevolver,
        Self::RetornoAguardando,
        Self::UltimaMovimentacao,
        Self::Marcadores,
    ];

    /// `name` da caixa de seleção na tela de configuração.
    pub(crate) fn campo(self) -> &'static str {
        match self {
            Self::Atribuicao => "chkSinAtribuicao",
            Self::Anotacao => "chkSinAnotacao",
            Self::TipoProcesso => "chkSinTipoProcesso",
            Self::Especificacao => "chkSinEspecificacao",
            Self::Prioridade => "chkSinNivelPrioritarios",
            Self::Interessados => "chkSinInteressados",
            Self::Observacao => "chkSinObservacao",
            Self::ControlePrazo => "chkSinControlePrazo",
            Self::RetornoDevolver => "chkSinRetornoDevolver",
            Self::RetornoAguardando => "chkSinRetornoAguardando",
            Self::UltimaMovimentacao => "chkSinUltimaMovimentacao",
            Self::Marcadores => "chkSinMarcadores",
        }
    }

    /// Texto do `label for=` da opção.
    pub(crate) fn rotulo(self) -> &'static str {
        match self {
            Self::Atribuicao => "Atribuição",
            Self::Anotacao => "Anotação",
            Self::TipoProcesso => "Tipo do Processo",
            Self::Especificacao => "Especificação",
            Self::Prioridade => "Prioridade",
            Self::Interessados => "Interessados",
            Self::Observacao => "Observação",
            Self::ControlePrazo => "Controle de Prazo",
            Self::RetornoDevolver => "Para Devolver",
            Self::RetornoAguardando => "Aguardando Retorno",
            Self::UltimaMovimentacao => "Última Movimentação na Unidade",
            Self::Marcadores => "Marcadores",
        }
    }

    /// Cabeçalho observado na tabela D.
    pub(crate) fn cabecalho(self) -> CabecalhoDaColuna {
        use CabecalhoDaColuna::{Icone, Texto};
        match self {
            Self::Atribuicao => Texto("Atribuição"),
            Self::Anotacao => Texto("Anotação"),
            Self::TipoProcesso => Texto("Tipo"),
            Self::Especificacao => Texto("Especificação"),
            Self::Prioridade => Texto("Prioridade"),
            Self::Interessados => Texto("Interessados"),
            Self::Observacao => Texto("Observação"),
            Self::ControlePrazo => Icone("Controle de Prazo"),
            Self::RetornoDevolver => Icone("Para Devolver"),
            Self::RetornoAguardando => Icone("Aguardando Retorno"),
            Self::UltimaMovimentacao => Texto("Última Mov."),
            Self::Marcadores => Texto("Marcadores"),
        }
    }

    /// Chave estável em `colunas` e `colunas_ausentes`.
    pub(crate) fn chave(self) -> &'static str {
        match self {
            Self::Atribuicao => "atribuicao",
            Self::Anotacao => "anotacao",
            Self::TipoProcesso => "tipo",
            Self::Especificacao => "especificacao",
            Self::Prioridade => "prioridade",
            Self::Interessados => "interessados",
            Self::Observacao => "observacao",
            Self::ControlePrazo => "controle_de_prazo",
            Self::RetornoDevolver => "para_devolver",
            Self::RetornoAguardando => "aguardando_retorno",
            Self::UltimaMovimentacao => "ultima_movimentacao",
            Self::Marcadores => "marcadores",
        }
    }

    /// Célula com itens separados (`div.divItemCelula`), observada em
    /// Interessados (PEN1). As demais são texto único.
    pub(crate) fn tem_itens(self) -> bool {
        self == Self::Interessados
    }

    /// Conta para `colunas_ausentes`: as nove com cabeçalho de texto
    /// (decisão do proprietário, 02/11/2026: máximo = 9 + Processo).
    pub(crate) fn tem_cabecalho_de_texto(self) -> bool {
        matches!(self.cabecalho(), CabecalhoDaColuna::Texto(_))
    }

    /// Coluna pelo rótulo do cabeçalho (texto colapsado ou rótulo do ícone).
    pub(crate) fn pelo_cabecalho(rotulo: &str) -> Option<Self> {
        Self::TODAS
            .into_iter()
            .find(|coluna| match coluna.cabecalho() {
                CabecalhoDaColuna::Texto(texto) | CabecalhoDaColuna::Icone(texto) => {
                    texto == rotulo
                }
            })
    }

    /// Coluna pela chave estável (`atribuicao`, `tipo`, …).
    pub(crate) fn pela_chave(chave: &str) -> Option<Self> {
        Self::TODAS
            .into_iter()
            .find(|coluna| coluna.chave() == chave)
    }

    pub(crate) fn pelo_campo(nome: &str) -> Option<Self> {
        Self::TODAS
            .into_iter()
            .find(|coluna| coluna.campo() == nome)
    }
}

/// Uma opção do catálogo: se a tela a publica e se está marcada (`checked`).
/// Opção não publicada (`disponivel: false`, como Prioridade na `UnB` 4.0.9)
/// tem `marcada: false`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OpcaoDoDetalhe {
    pub(crate) coluna: ColunaDoDetalhe,
    pub(crate) disponivel: bool,
    pub(crate) marcada: bool,
}

/// As 12 opções do catálogo, na ordem de [`ColunaDoDetalhe::TODAS`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NivelDeDetalhe {
    pub(crate) opcoes: Vec<OpcaoDoDetalhe>,
}

impl NivelDeDetalhe {
    /// Colunas marcadas, na ordem do catálogo.
    pub(crate) fn marcadas(&self) -> Vec<ColunaDoDetalhe> {
        self.opcoes
            .iter()
            .filter(|opcao| opcao.marcada)
            .map(|opcao| opcao.coluna)
            .collect()
    }

    pub(crate) fn json(&self) -> serde_json::Value {
        let opcoes: Vec<_> = self
            .opcoes
            .iter()
            .map(|opcao| {
                let (cabecalho, tipo) = match opcao.coluna.cabecalho() {
                    CabecalhoDaColuna::Texto(texto) => (texto, "texto"),
                    CabecalhoDaColuna::Icone(texto) => (texto, "icone"),
                };
                serde_json::json!({
                    "coluna": opcao.coluna.chave(),
                    "campo": opcao.coluna.campo(),
                    "rotulo": opcao.coluna.rotulo(),
                    "cabecalho": cabecalho,
                    "cabecalho_tipo": tipo,
                    "disponivel": opcao.disponivel,
                    "marcada": opcao.marcada,
                })
            })
            .collect();
        serde_json::Value::Array(opcoes)
    }
}

#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
pub(crate) mod tests_alvo;

#[cfg(test)]
pub(crate) mod tests_salvar;
