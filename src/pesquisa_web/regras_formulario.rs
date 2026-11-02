//! Regras do `onsubmit` de `frmPesquisaProtocolo`, escritas à mão a partir do
//! JavaScript servido em `unb`, `pen1`, `ufrj` e `go` (GET de
//! `protocolo_pesquisar` das cassettes `processo-busca-*`). Nada de JS é
//! executado ou interpretado aqui. Evidência: docs/decisoes/processo-buscar.md.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroRegraDoFormularioDePesquisa {
    OpcaoDeProcessosAusenteNaTela,
    OpcoesDeProcessosRepetidasNaTela,
    OpcaoDeProcessosSemValorNaTela,
    OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm,
    AssinanteOcultoPreenchidoNaPesquisaEmProcessos,
    SeletorDeOrgaoDePesquisaAusenteNaTela,
    SeletoresDeOrgaoDePesquisaRepetidosNaTela,
    SeletorDeOrgaoDePesquisaSemMultiplaEscolha,
    OrgaoDePesquisaSemOpcaoSelecionadaNaTela,
    OpcaoDeOrgaoSelecionadaSemValorNaTela,
    InicioDaPaginacaoDiferenteDeZeroNaTela,
    EntidadeHtmlDesconhecidaNaRegra,
}

impl ErroRegraDoFormularioDePesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::OpcaoDeProcessosAusenteNaTela => "opcao_de_processos_ausente_na_tela",
            Self::OpcoesDeProcessosRepetidasNaTela => "opcoes_de_processos_repetidas_na_tela",
            Self::OpcaoDeProcessosSemValorNaTela => "opcao_de_processos_sem_valor_na_tela",
            Self::OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm => {
                "opcao_de_processos_com_nome_diferente_de_rdo_pesquisar_em"
            }
            Self::AssinanteOcultoPreenchidoNaPesquisaEmProcessos => {
                "assinante_oculto_preenchido_na_pesquisa_em_processos"
            }
            Self::SeletorDeOrgaoDePesquisaAusenteNaTela => {
                "seletor_de_orgao_de_pesquisa_ausente_na_tela"
            }
            Self::SeletoresDeOrgaoDePesquisaRepetidosNaTela => {
                "seletores_de_orgao_de_pesquisa_repetidos_na_tela"
            }
            Self::SeletorDeOrgaoDePesquisaSemMultiplaEscolha => {
                "seletor_de_orgao_de_pesquisa_sem_multipla_escolha"
            }
            Self::OrgaoDePesquisaSemOpcaoSelecionadaNaTela => {
                "orgao_de_pesquisa_sem_opcao_selecionada_na_tela"
            }
            Self::OpcaoDeOrgaoSelecionadaSemValorNaTela => {
                "opcao_de_orgao_selecionada_sem_valor_na_tela"
            }
            Self::InicioDaPaginacaoDiferenteDeZeroNaTela => {
                "inicio_da_paginacao_diferente_de_zero_na_tela"
            }
            Self::EntidadeHtmlDesconhecidaNaRegra => "entidade_html_desconhecida_na_regra",
        }
    }
}

/// Valores que o `onsubmit` exige e que a operação lê da tela servida.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RegrasDaTela {
    pub(crate) opcao_de_processos: String,
    pub(crate) orgaos_selecionados: Vec<String>,
}

pub(crate) fn ler_regras_da_tela(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<RegrasDaTela, ErroRegraDoFormularioDePesquisa> {
    let _ = (form, parser);
    unimplemented!("CONTRACT shim: regras do onsubmit")
}
