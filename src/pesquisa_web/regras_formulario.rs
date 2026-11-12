//! Regras do `onsubmit` de `frmPesquisaProtocolo`, escritas à mão a partir do
//! JavaScript servido em `unb`, `pen1`, `ufrj` e `go` (GET de
//! `protocolo_pesquisar` das cassettes `processo-busca-*`). Nada de JS é
//! executado ou interpretado aqui. Evidência: docs/decisoes/processo-buscar.md.

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroRegraDoFormularioDePesquisa {
    OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm,
    AssinanteOcultoPreenchidoNaPesquisaEmProcessos,
    SeletorDeOrgaoDePesquisaSemMultiplaEscolha,
    OrgaoDePesquisaSemOpcaoSelecionadaNaTela,
    InicioDaPaginacaoDiferenteDeZeroNaTela,
    /// O termo tem mais unidades UTF-16 que o `maxlength` servido, que o
    /// navegador não deixaria digitar.
    TermoAcimaDoMaxlengthDaTela {
        maxlength: u32,
    },
    /// `maxlength` do termo servido fora de um inteiro decimal não negativo.
    MaxlengthDoTermoIlegivelNaTela,
    /// `maxlength` decimal que não cabe em `u32`.
    MaxlengthDoTermoNaoCabeEmU32NaTela(#[source] std::num::ParseIntError),
    /// O campo `txtProtocoloPesquisa` não existe na tela do formulário.
    CampoDoTermoAusenteNaTela,
    /// O campo do termo existe, mas a tela não serve `maxlength` nele.
    MaxlengthDoTermoAusenteNaTela,
    OpcaoDeDocumentosComNomeDiferenteDeRdoPesquisarEm,
    /// Nenhuma das duas servida marcada: `onSubmitForm()` não envia.
    NenhumTipoDeDocumentoMarcadoNaTela,
    AssinanteOcultoPreenchidoNaPesquisaEmDocumentos,
    CaixaDeTipoDeDocumentoAusenteNaTela,
    CaixaDeTipoDeDocumentoSemValorNaTela,
    CaixasDeTipoDeDocumentoRepetidasNaTela,
    CamposDoTermoRepetidosNaTela,
    OpcaoDeDocumentosAusenteNaTela,
    OpcaoDeDocumentosSemValorNaTela,
    OpcaoDeOrgaoSelecionadaSemValorNaTela,
    OpcaoDeProcessosAusenteNaTela,
    OpcaoDeProcessosSemValorNaTela,
    OpcoesDeDocumentosRepetidasNaTela,
    OpcoesDeProcessosRepetidasNaTela,
    SeletorDeOrgaoDePesquisaAusenteNaTela,
    SeletoresDeOrgaoDePesquisaRepetidosNaTela,
}

/// Onde pesquisar: o rádio `rdoPesquisarEm` que a operação marca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PesquisarEm {
    /// `#optProcessos` (`ies processo buscar`).
    Processos,
    /// `#optDocumentos` (`ies documento buscar`), com as caixas de tipo.
    Documentos,
}

/// Valores que o `onsubmit` exige e que a operação lê da tela servida.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RegrasDaTela {
    /// Valor servido do rádio da opção pedida.
    pub(crate) opcao: String,
    /// Em documentos: as caixas de tipo servidas marcadas (nome, valor).
    pub(crate) caixas_de_documento: Vec<(&'static str, String)>,
    pub(crate) orgaos_selecionados: Vec<String>,
    /// `maxlength` servido no campo do termo, sempre publicado pela tela.
    pub(crate) maxlength_do_termo: u32,
}

fn eh(tag: crate::tela_html::Elemento<'_>, elemento: &str) -> bool {
    tag.nome().eq_ignore_ascii_case(elemento)
}

/// `onSubmitForm()` exige rádio marcado; com `P`, a operação usa
/// `#optProcessos` como servido; com `D`, `#optDocumentos` (e as caixas).
fn opcao(
    tags: &[crate::tela_html::Elemento<'_>],
    em: PesquisarEm,
) -> Result<String, ErroRegraDoFormularioDePesquisa> {
    let (id, ausente, repetidas, sem_valor, outro_nome) = match em {
        PesquisarEm::Processos => (
            "optProcessos",
            ErroRegraDoFormularioDePesquisa::OpcaoDeProcessosAusenteNaTela,
            ErroRegraDoFormularioDePesquisa::OpcoesDeProcessosRepetidasNaTela,
            ErroRegraDoFormularioDePesquisa::OpcaoDeProcessosSemValorNaTela,
            ErroRegraDoFormularioDePesquisa::OpcaoDeProcessosComNomeDiferenteDeRdoPesquisarEm,
        ),
        PesquisarEm::Documentos => (
            "optDocumentos",
            ErroRegraDoFormularioDePesquisa::OpcaoDeDocumentosAusenteNaTela,
            ErroRegraDoFormularioDePesquisa::OpcoesDeDocumentosRepetidasNaTela,
            ErroRegraDoFormularioDePesquisa::OpcaoDeDocumentosSemValorNaTela,
            ErroRegraDoFormularioDePesquisa::OpcaoDeDocumentosComNomeDiferenteDeRdoPesquisarEm,
        ),
    };
    let mut achadas = Vec::new();
    for tag in tags.iter().filter(|t| eh(**t, "input")) {
        if tag.atributo("id") == Some(id) {
            achadas.push(*tag);
        }
    }
    let radio = match achadas.as_slice() {
        [] => return Err(ausente),
        [radio] => *radio,
        _ => return Err(repetidas),
    };
    if radio.atributo("name") != Some("rdoPesquisarEm") {
        return Err(outro_nome);
    }
    radio.atributo("value").map(str::to_owned).ok_or(sem_valor)
}

/// Caixas que `onSubmitForm()` lê por id com `D`: cada uma servida uma vez,
/// com `name` igual ao id; vão as marcadas, com o valor servido, como o
/// navegador enviaria. Nenhuma marcada: `onSubmitForm()` recusaria.
const CAIXAS_DE_DOCUMENTO: [&str; 2] = ["chkSinDocumentosGerados", "chkSinDocumentosRecebidos"];

fn caixas_de_documento(
    tags: &[crate::tela_html::Elemento<'_>],
) -> Result<Vec<(&'static str, String)>, ErroRegraDoFormularioDePesquisa> {
    let mut marcadas = Vec::new();
    for caixa in CAIXAS_DE_DOCUMENTO {
        let mut achadas = Vec::new();
        for tag in tags.iter().filter(|t| eh(**t, "input")) {
            let id = tag.atributo("id").map(str::to_owned);
            let nome = tag.atributo("name").map(str::to_owned);
            if id.as_deref() == Some(caixa) || nome.as_deref() == Some(caixa) {
                achadas.push((*tag, id, nome));
            }
        }
        let (tag, id, nome) = match achadas.as_slice() {
            [] => {
                return Err(ErroRegraDoFormularioDePesquisa::CaixaDeTipoDeDocumentoAusenteNaTela);
            }
            [achada] => achada,
            _ => {
                return Err(
                    ErroRegraDoFormularioDePesquisa::CaixasDeTipoDeDocumentoRepetidasNaTela,
                );
            }
        };
        if id.as_deref() != Some(caixa) || nome.as_deref() != Some(caixa) {
            return Err(ErroRegraDoFormularioDePesquisa::CaixaDeTipoDeDocumentoAusenteNaTela);
        }
        if tag.presente("checked") {
            let valor = tag
                .atributo("value")
                .map(str::to_owned)
                .ok_or(ErroRegraDoFormularioDePesquisa::CaixaDeTipoDeDocumentoSemValorNaTela)?;
            marcadas.push((caixa, valor));
        }
    }
    if marcadas.is_empty() {
        return Err(ErroRegraDoFormularioDePesquisa::NenhumTipoDeDocumentoMarcadoNaTela);
    }
    Ok(marcadas)
}

/// Com `P`, `tratarCamposDocumento()` esvazia `hdnIdAssinante`; com `D`, o
/// servido preenchido filtraria em silêncio. `navegar()` só reescreve
/// `hdnInicio` na paginação. Ocultos saem como servidos.
fn conferir_ocultos(
    tags: &[crate::tela_html::Elemento<'_>],
    em: PesquisarEm,
) -> Result<(), ErroRegraDoFormularioDePesquisa> {
    for tag in tags.iter().filter(|t| eh(**t, "input")) {
        let valor = || tag.atributo("value").unwrap_or_default();
        match tag.atributo("name") {
            Some("hdnIdAssinante") if !valor().is_empty() => {
                return Err(match em {
                    PesquisarEm::Processos => ErroRegraDoFormularioDePesquisa::AssinanteOcultoPreenchidoNaPesquisaEmProcessos,
                    PesquisarEm::Documentos => ErroRegraDoFormularioDePesquisa::AssinanteOcultoPreenchidoNaPesquisaEmDocumentos,
                });
            }
            Some("hdnInicio") if valor() != "0" => {
                return Err(
                    ErroRegraDoFormularioDePesquisa::InicioDaPaginacaoDiferenteDeZeroNaTela,
                );
            }
            _ => {}
        }
    }
    Ok(())
}

/// `onSubmitForm()` exige órgão em `selOrgaoPesquisa[]`: as opções com
/// `selected` servidas, na ordem do documento; nenhuma é inventada.
/// O `select` é achado pelo `name` (o nome serializado no POST), não pelo
/// `id`: a regra continua mais precisa mesmo sem o defeito do tl 0.7.8.
fn orgaos_selecionados(
    tags: &[crate::tela_html::Elemento<'_>],
) -> Result<Vec<String>, ErroRegraDoFormularioDePesquisa> {
    let mut achados = Vec::new();
    for tag in tags.iter().filter(|t| eh(**t, "select")) {
        if tag.atributo("name") == Some("selOrgaoPesquisa[]") {
            achados.push(*tag);
        }
    }
    let select = match achados.as_slice() {
        [] => {
            return Err(ErroRegraDoFormularioDePesquisa::SeletorDeOrgaoDePesquisaAusenteNaTela);
        }
        [select] => *select,
        _ => {
            return Err(ErroRegraDoFormularioDePesquisa::SeletoresDeOrgaoDePesquisaRepetidosNaTela);
        }
    };
    if !select.presente("multiple") {
        return Err(ErroRegraDoFormularioDePesquisa::SeletorDeOrgaoDePesquisaSemMultiplaEscolha);
    }
    let descendentes: Vec<_> = select.descendentes().collect();
    // Como no navegador: opção desabilitada, própria ou por `optgroup
    // disabled`, não é serializada mesmo se servida com `selected`.
    let mut desabilitadas = Vec::new();
    for grupo in descendentes.iter().filter(|t| eh(**t, "optgroup")) {
        if grupo.presente("disabled") {
            desabilitadas.extend(grupo.descendentes());
        }
    }
    let mut valores = Vec::new();
    for opcao in descendentes {
        if !eh(opcao, "option") || opcao.presente("disabled") || desabilitadas.contains(&opcao) {
            continue;
        }
        if opcao.presente("selected") {
            valores.push(
                opcao.atributo("value").map(str::to_owned).ok_or(
                    ErroRegraDoFormularioDePesquisa::OpcaoDeOrgaoSelecionadaSemValorNaTela,
                )?,
            );
        }
    }
    if valores.is_empty() {
        return Err(ErroRegraDoFormularioDePesquisa::OrgaoDePesquisaSemOpcaoSelecionadaNaTela);
    }
    Ok(valores)
}

/// `maxlength` de `txtProtocoloPesquisa` como servido (50 nas quatro
/// instalações, sem presumir esse valor). Só dígitos ASCII são aceitos,
/// mais estrito que o navegador, que ignora o atributo inválido.
fn maxlength_do_termo(
    tags: &[crate::tela_html::Elemento<'_>],
) -> Result<u32, ErroRegraDoFormularioDePesquisa> {
    let mut campos = Vec::new();
    for tag in tags.iter().filter(|t| eh(**t, "input")) {
        if tag.atributo("name") == Some("txtProtocoloPesquisa") {
            campos.push(*tag);
        }
    }
    let campo = match campos.as_slice() {
        [] => return Err(ErroRegraDoFormularioDePesquisa::CampoDoTermoAusenteNaTela),
        [campo] => *campo,
        _ => return Err(ErroRegraDoFormularioDePesquisa::CamposDoTermoRepetidosNaTela),
    };
    let Some(valor) = campo.atributo("maxlength").map(str::to_owned) else {
        return Err(ErroRegraDoFormularioDePesquisa::MaxlengthDoTermoAusenteNaTela);
    };
    if valor.is_empty() || !valor.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroRegraDoFormularioDePesquisa::MaxlengthDoTermoIlegivelNaTela);
    }
    valor
        .parse()
        .map_err(ErroRegraDoFormularioDePesquisa::MaxlengthDoTermoNaoCabeEmU32NaTela)
}

/// Lê, do `<form>` já localizado, os valores que o `onsubmit` exige.
pub(crate) fn ler_regras_da_tela(
    form: crate::tela_html::Elemento<'_>,
    em: PesquisarEm,
) -> Result<RegrasDaTela, ErroRegraDoFormularioDePesquisa> {
    let tags: Vec<_> = form.descendentes().collect();
    let opcao = opcao(&tags, em)?;
    let caixas_de_documento = match em {
        PesquisarEm::Processos => Vec::new(),
        PesquisarEm::Documentos => caixas_de_documento(&tags)?,
    };
    conferir_ocultos(&tags, em)?;
    let orgaos_selecionados = orgaos_selecionados(&tags)?;
    let maxlength_do_termo = maxlength_do_termo(&tags)?;
    Ok(RegrasDaTela {
        opcao,
        caixas_de_documento,
        orgaos_selecionados,
        maxlength_do_termo,
    })
}
