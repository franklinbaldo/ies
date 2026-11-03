//! Árvore do processo (`procedimento_visualizar`), lida sem regex e sem
//! executar JavaScript: só as linhas inteiras
//! `Nos[i] = new infraArvoreNo(a0,…,a15);` servidas no `script` inline.
//! Estrutura em docs/como-o-sei-funciona/arvore-do-processo.md; contrato em
//! contracts/cli/processo-obter-arvore.md. Nesta fatia: `PROCESSO` e
//! `DOCUMENTO` do primeiro nível; `PASTA` e qualquer outro tipo são recusa.
pub(crate) mod endereco;
mod literais;
mod tela;

#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
mod tests_literais;
#[cfg(test)]
pub(crate) mod tests_tela;

use literais::{Literal, ler_declaracao};
pub(crate) use tela::{ErroTelaDoProcesso, ler_tela_do_processo};

/// O nó `PROCESSO`: id (a1), NUP (a5, igual a a15) e tipo (a6, a dica).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoNaArvore {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: String,
}

/// Um `DOCUMENTO` do primeiro nível: id (a1), rótulo (a5) e número SEI
/// (a15, que pode vir `null`). A dica (a6) não é guardada: em e-mails traz
/// remetente e destinatários.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoNaArvore {
    pub(crate) id: String,
    pub(crate) rotulo: String,
    pub(crate) numero_sei: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArvoreLida {
    pub(crate) processo: ProcessoNaArvore,
    pub(crate) documentos: Vec<DocumentoNaArvore>,
}

/// Uma variante por fato verificável; nenhum valor servido é guardado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroArvore {
    DeclaracaoDeNoForaDaFormaObservada,
    ArgumentoDeNoNaoLiteral,
    TextoDeNoSemFechamento,
    EscapeDeNoNaoReconhecido,
    QuantidadeDeArgumentosDoNoDiferenteDaObservada,
    ArgumentoDeNoComTipoDiferenteDoObservado,
    IndiceDeNoForaDaSequencia,
    TipoDeNoDaArvoreNaoSuportado,
    NoDoProcessoAusenteNaArvore,
    NosDoProcessoRepetidosNaArvore,
    NoDoProcessoComPai,
    NupDoNoDoProcessoInconsistente,
    IdDeNoNaoDecimal,
    DocumentoComPaiDiferenteDoProcesso,
    DocumentoRepetidoNaArvore,
    IdentidadeDoProcessoDiferenteDaPedida,
    IdDoProcessoNaArvoreDiferenteDoAberto,
}

impl ErroArvore {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::DeclaracaoDeNoForaDaFormaObservada => "declaracao_de_no_fora_da_forma_observada",
            Self::ArgumentoDeNoNaoLiteral => "argumento_de_no_nao_literal",
            Self::TextoDeNoSemFechamento => "texto_de_no_sem_fechamento",
            Self::EscapeDeNoNaoReconhecido => "escape_de_no_nao_reconhecido",
            Self::QuantidadeDeArgumentosDoNoDiferenteDaObservada => {
                "quantidade_de_argumentos_do_no_diferente_da_observada"
            }
            Self::ArgumentoDeNoComTipoDiferenteDoObservado => {
                "argumento_de_no_com_tipo_diferente_do_observado"
            }
            Self::IndiceDeNoForaDaSequencia => "indice_de_no_fora_da_sequencia",
            Self::TipoDeNoDaArvoreNaoSuportado => "tipo_de_no_da_arvore_nao_suportado",
            Self::NoDoProcessoAusenteNaArvore => "no_do_processo_ausente_na_arvore",
            Self::NosDoProcessoRepetidosNaArvore => "nos_do_processo_repetidos_na_arvore",
            Self::NoDoProcessoComPai => "no_do_processo_com_pai",
            Self::NupDoNoDoProcessoInconsistente => "nup_do_no_do_processo_inconsistente",
            Self::IdDeNoNaoDecimal => "id_de_no_nao_decimal",
            Self::DocumentoComPaiDiferenteDoProcesso => "documento_com_pai_diferente_do_processo",
            Self::DocumentoRepetidoNaArvore => "documento_repetido_na_arvore",
            Self::IdentidadeDoProcessoDiferenteDaPedida => {
                "identidade_do_processo_diferente_da_pedida"
            }
            Self::IdDoProcessoNaArvoreDiferenteDoAberto => {
                "id_do_processo_na_arvore_diferente_do_aberto"
            }
        }
    }

    /// A árvore é de outro processo: um palpite do índice errou.
    pub(crate) fn diverge_do_pedido(self) -> bool {
        matches!(
            self,
            Self::IdentidadeDoProcessoDiferenteDaPedida
                | Self::IdDoProcessoNaArvoreDiferenteDoAberto
        )
    }
}

/// Argumentos por nó, observados na `UnB` 4.0.9 e em GO.
const ARGUMENTOS: usize = 16;

/// Lê os nós da árvore e confere a identidade: o NUP do nó `PROCESSO` tem
/// de ser **igual** ao pedido (comparação opaca) e o id igual ao aberto.
pub(crate) fn ler_arvore(
    html: &str,
    nup_pedido: &str,
    id_aberto: &str,
) -> Result<ArvoreLida, ErroArvore> {
    let mut processo: Option<ProcessoNaArvore> = None;
    // Documentos com o pai declarado; o pai é conferido no fim, porque o
    // nó do processo vem antes, mas a regra não depende da ordem.
    let mut documentos: Vec<(String, DocumentoNaArvore)> = Vec::new();
    let mut proximo = 0usize;
    for linha in html.split('\n') {
        let Some((indice, argumentos)) = ler_declaracao(linha)? else {
            continue;
        };
        if indice != proximo {
            return Err(ErroArvore::IndiceDeNoForaDaSequencia);
        }
        proximo += 1;
        let argumentos: [Literal; ARGUMENTOS] = argumentos
            .try_into()
            .map_err(|_| ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada)?;
        match texto(&argumentos[0])? {
            "PROCESSO" => {
                if processo.is_some() {
                    return Err(ErroArvore::NosDoProcessoRepetidosNaArvore);
                }
                processo = Some(ler_processo(&argumentos)?);
            }
            "DOCUMENTO" => {
                let (pai, documento) = ler_documento(&argumentos)?;
                if documentos.iter().any(|(_, d)| d.id == documento.id) {
                    return Err(ErroArvore::DocumentoRepetidoNaArvore);
                }
                documentos.push((pai, documento));
            }
            _ => return Err(ErroArvore::TipoDeNoDaArvoreNaoSuportado),
        }
    }
    let processo = processo.ok_or(ErroArvore::NoDoProcessoAusenteNaArvore)?;
    if documentos.iter().any(|(pai, _)| *pai != processo.id) {
        return Err(ErroArvore::DocumentoComPaiDiferenteDoProcesso);
    }
    if processo.nup != nup_pedido {
        return Err(ErroArvore::IdentidadeDoProcessoDiferenteDaPedida);
    }
    if processo.id != id_aberto {
        return Err(ErroArvore::IdDoProcessoNaArvoreDiferenteDoAberto);
    }
    Ok(ArvoreLida {
        processo,
        documentos: documentos.into_iter().map(|(_, d)| d).collect(),
    })
}

fn texto(literal: &Literal) -> Result<&str, ErroArvore> {
    match literal {
        Literal::Texto(texto) => Ok(texto),
        _ => Err(ErroArvore::ArgumentoDeNoComTipoDiferenteDoObservado),
    }
}

fn texto_ou_nulo(literal: &Literal) -> Result<Option<&str>, ErroArvore> {
    match literal {
        Literal::Nulo => Ok(None),
        outro => texto(outro).map(Some),
    }
}

fn id(literal: &Literal) -> Result<String, ErroArvore> {
    let id = texto(literal)?;
    if id.is_empty() || id.len() > 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroArvore::IdDeNoNaoDecimal);
    }
    Ok(id.to_owned())
}

fn ler_processo(a: &[Literal; ARGUMENTOS]) -> Result<ProcessoNaArvore, ErroArvore> {
    let id = id(&a[1])?;
    if texto_ou_nulo(&a[2])?.is_some() {
        return Err(ErroArvore::NoDoProcessoComPai);
    }
    let nup = texto(&a[5])?;
    if texto(&a[15])? != nup {
        return Err(ErroArvore::NupDoNoDoProcessoInconsistente);
    }
    Ok(ProcessoNaArvore {
        id,
        nup: nup.to_owned(),
        tipo: texto(&a[6])?.to_owned(),
    })
}

/// O pai (a2) e o documento; a dica (a6) não é lida.
fn ler_documento(a: &[Literal; ARGUMENTOS]) -> Result<(String, DocumentoNaArvore), ErroArvore> {
    let documento = DocumentoNaArvore {
        id: id(&a[1])?,
        rotulo: texto(&a[5])?.to_owned(),
        numero_sei: texto_ou_nulo(&a[15])?.map(str::to_owned),
    };
    Ok((id(&a[2])?, documento))
}
