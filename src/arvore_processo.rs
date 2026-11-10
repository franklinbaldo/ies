//! Árvore do processo (`procedimento_visualizar`), lida sem regex e sem
//! executar JavaScript: só as linhas inteiras
//! `Nos[i] = new infraArvoreNo(…);` servidas no `script` inline, e, nas
//! pastas, `Pastas[k]['protocolos']` e a barra (`.acoes`) dos documentos.
//! Estrutura em docs/como-o-sei-funciona/arvore-do-processo.md; contrato em
//! contracts/cli/processo-obter-arvore.md. Tipos aceitos: `PROCESSO`,
//! `PROCESSO_ANEXADO`, `DOCUMENTO`, `DOCUMENTO_MOVIDO`, `PASTA` e
//! `AGUARDE`; qualquer outro é recusa.
mod acompanhar;
mod andamento;
mod anexados;
mod anexar;
mod assinatura;
mod atribuir;
mod barra_do_documento;
mod barra_do_processo;
mod ciencia;
mod concluir;
mod conferencia_parcial;
mod conteudo;
pub(crate) mod endereco;
mod enviar;
mod erro;
mod literais;
mod movidos;
mod natureza;
mod pastas;
mod reabrir;
mod relacionar;
mod situacao;
mod sobrestar;
mod tela;
mod vinculo;

#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
pub(crate) mod tests_anexado_a;
#[cfg(test)]
pub(crate) mod tests_anexados;
#[cfg(test)]
pub(crate) mod tests_assinatura;
#[cfg(test)]
mod tests_barra;
#[cfg(test)]
pub(crate) mod tests_barra_documento;
#[cfg(test)]
mod tests_barra_marcadores;
#[cfg(test)]
pub(crate) mod tests_concluir;
#[cfg(test)]
pub(crate) mod tests_conteudo;
#[cfg(test)]
mod tests_literais;
#[cfg(test)]
pub(crate) mod tests_movido_presente;
#[cfg(test)]
pub(crate) mod tests_movidos;
#[cfg(test)]
mod tests_natureza;
#[cfg(test)]
pub(crate) mod tests_pastas;
#[cfg(test)]
mod tests_pastas_concatenacao;
#[cfg(test)]
mod tests_pastas_integridade;
#[cfg(test)]
pub(crate) mod tests_reabrir;
#[cfg(test)]
mod tests_situacao;
#[cfg(test)]
mod tests_sobrestar;
#[cfg(test)]
pub(crate) mod tests_tela;

use crate::erro_html::ErroHtml;
pub(crate) use acompanhar::{ErroAcompanharNaBarra, ler_acompanhar_da_barra};
pub(crate) use andamento::{ErroAndamentoNaBarra, ler_andamento_da_barra};
use anexados::{AnexadoLido, conferir_anexados, ler_anexado};
pub(crate) use anexar::{ErroAnexarNaBarra, ler_anexar_da_barra};
pub(crate) use assinatura::{
    ACAO_DE_ASSINAR, Assinante, ErroLinkDeAssinar, ler_assinatura_na_arvore, ler_link_de_assinar,
    tela_do_documento,
};
pub(crate) use atribuir::{ErroAtribuirNaBarra, ler_atribuir_da_barra};
pub(crate) use barra_do_documento::barra_do_no;
pub(crate) use barra_do_documento::{
    BarraDoDocumento, EdicaoNaBarra, ErroBarraDoDocumento, ler_barra_do_documento,
};
pub(crate) use barra_do_processo::ler_link_das_anotacoes;
pub(crate) use barra_do_processo::ler_link_dos_comentarios;
pub(crate) use barra_do_processo::{
    EnderecoDoIncluirDocumento, ErroLinkIncluirDocumento, ler_link_de_incluir_documento,
};
pub(crate) use barra_do_processo::{
    ErroLinkPrazo, ler_link_das_propriedades, ler_link_do_prazo, ler_link_dos_marcadores,
};
pub(crate) use ciencia::niveis_js;
pub(crate) use ciencia::{
    ErroCienciaNaArvore, ciencia_da_barra, ciencia_do_documento_na_barra, lista_do_documento,
    lista_do_processo, url_da_ciencia, url_da_ciencia_do_documento,
};
pub(crate) use concluir::{
    ACAO_DE_CONCLUIR, ConcluirNaBarra, EnderecoDeEscrita, ErroConcluirNaBarra,
    ErroFuncaoDeConcluir, ler_concluir_da_barra, ler_funcao_de_concluir,
};
pub(crate) use conteudo::{
    ConteudoNaArvore, EnderecoDoConteudo, ErroConteudoNaArvore, ler_conteudo_na_arvore,
};
pub(crate) use endereco::conferir_endereco_relativo;
pub(crate) use enviar::{ACAO_DE_ENVIAR, ErroEnviarNaBarra, ler_enviar_da_barra};
pub(crate) use erro::ErroArvore;
use literais::{Literal, ler_barra, ler_declaracao, ler_protocolos};
pub(crate) use movidos::ler_destinos_dos_movidos;
use movidos::{MovidoLido, conferir_movidos, ler_movido};
pub(crate) use natureza::NaturezaDoDocumento;
pub(crate) use pastas::ler_acao_de_abrir_pastas;
use pastas::{NoDePasta, Pai};
pub(crate) use reabrir::{
    ErroEnderecoDeReabrir, FormaDaReabertura, ReabrirNaBarra, ler_endereco_de_reabrir,
    ler_reabrir_da_barra,
};
pub(crate) use relacionar::{ErroRelacionarNaBarra, ler_relacionar_da_barra};
pub(crate) use situacao::{AbertoEm, MotivoAbertoEm};
#[cfg(test)]
pub(crate) use situacao::{Atribuicao, UnidadeAberta, UnidadeSobrestada};
pub(crate) use sobrestar::{
    ErroAcaoSobrestar, ErroEnderecoDeRetomar, ler_endereco_de_retomar, ler_retomar_da_barra,
    ler_sobrestar_da_barra,
};
pub(crate) use tela::{
    EnderecoDaArvore, ErroTelaDoProcesso, TelaDoProcesso, ler_tela_do_documento,
    ler_tela_do_processo,
};
pub(crate) use vinculo::ler_vinculo_de_anexacao;

/// O nó `PROCESSO` (ou um `PROCESSO_ANEXADO`): id (a1), NUP (a5, igual a
/// a15) e tipo (a6, a dica).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoNaArvore {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: String,
}

/// Um `DOCUMENTO`: id (a1), rótulo (a5), número SEI (a15, que pode vir
/// `null`), a pasta (`PASTA<k>`) quando o pai (a2) é uma e a natureza
/// (a6 a a9). A dica (a6) não é guardada: em e-mails traz remetente e
/// destinatários.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoNaArvore {
    pub(crate) id: String,
    pub(crate) rotulo: String,
    pub(crate) numero_sei: Option<String>,
    pub(crate) pasta: Option<String>,
    pub(crate) natureza: NaturezaDoDocumento,
}

/// Uma `PASTA`: id do nó (`PASTA<k>`), rótulo (a5), quantos protocolos
/// `Pastas[k]['protocolos']` declara e como a pasta foi conferida (`None`:
/// fechada, não conferida nesta leitura).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PastaNaArvore {
    pub(crate) id: String,
    pub(crate) rotulo: String,
    pub(crate) total_declarado: usize,
    pub(crate) verificacao: Option<VerificacaoDaPasta>,
    /// Documentos conferidos pela identidade da barra.
    pub(crate) conferidos_por_identidade: usize,
    /// Documentos sem identidade na barra, que só entram na contagem.
    pub(crate) conferidos_por_contagem: usize,
}

/// Como os documentos de uma pasta foram conferidos com a declaração.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoDaPasta {
    /// Cada documento pelo `id_rel_protocolo_protocolo` da barra.
    PorIdentidade,
    /// Parte pela identidade, o resto (sem identidade na barra) pela contagem.
    Mista,
    /// Nenhum documento com identidade na barra: só a quantidade, com ids de
    /// documento únicos na árvore.
    PorContagem,
}

impl VerificacaoDaPasta {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::PorIdentidade => "por_identidade",
            Self::Mista => "mista",
            Self::PorContagem => "por_contagem",
        }
    }
}

/// A barra (`Nos[i].acoes`) de um documento de pasta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Barra {
    /// Nenhuma linha `.acoes` para o nó (nunca observado: recusa).
    NaoLida,
    /// Vazia (`''`) ou com links sem `id_rel_protocolo_protocolo`.
    SemIdentidade,
    Identidade(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArvoreLida {
    pub(crate) processo: ProcessoNaArvore,
    pub(crate) documentos: Vec<DocumentoNaArvore>,
    pub(crate) pastas: Vec<PastaNaArvore>,
    /// Alguma pasta veio fechada (com o nó `AGUARDE`): os documentos dela
    /// não estão nesta árvore.
    pub(crate) pastas_nao_carregadas: bool,
    /// Indica se a lista de documentos foi integralmente confirmada.
    pub(crate) documentos_incluidos: bool,
    /// Aviso emitido quando a leitura foi parcial por inconsistência de pastas.
    pub(crate) aviso_leitura_parcial: Option<ErroArvore>,
    /// Os `PROCESSO_ANEXADO` filhos do processo, na ordem da árvore. Os
    /// documentos deles não são servidos nesta árvore.
    pub(crate) anexados: Vec<ProcessoNaArvore>,
    /// Os `DOCUMENTO_MOVIDO` (movidos para outro processo), na ordem da
    /// árvore; não são documentos do processo.
    pub(crate) documentos_movidos: Vec<DocumentoMovido>,
    /// O pai, quando o processo aberto é ele mesmo um anexado (link do
    /// `.html` do nó raiz); preenchido por [`ler_vinculo_de_anexacao`].
    pub(crate) anexado_a: Option<VinculoDeAnexacao>,
    /// Onde o processo está aberto (o `.html` da raiz).
    pub(crate) aberto_em: AbertoEm,
}

/// O processo ao qual o aberto está anexado: NUP (texto do link), id e o
/// `procedimento_trabalhar` assinado dele (redigido no `Debug`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VinculoDeAnexacao {
    pub(crate) numero: String,
    pub(crate) id_procedimento: String,
    pub(crate) trabalhar: EnderecoDaArvore,
}

/// Um `DOCUMENTO_MOVIDO`: id composto opaco (a1), rótulo (a5), número SEI
/// (a15) e o destino, quando o `.html` do nó publica o link assinado dele
/// (preenchido por [`ler_destinos_dos_movidos`]). A dica (a6) não é guardada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoMovido {
    pub(crate) id: String,
    pub(crate) rotulo: String,
    pub(crate) numero_sei: Option<String>,
    pub(crate) destino: Option<DestinoDoMovido>,
    /// A primeira parte do id é o id de um `DOCUMENTO` desta árvore.
    pub(crate) presente_entre_os_documentos: bool,
}

/// O processo para onde o documento foi movido: NUP (texto do link), id e o
/// `procedimento_trabalhar` assinado (redigido no `Debug`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DestinoDoMovido {
    pub(crate) numero: String,
    pub(crate) id_procedimento: String,
    pub(crate) trabalhar: EnderecoDaArvore,
}

/// Argumentos por nó, observados na `UnB` 4.0.9 e em GO (`PROCESSO`,
/// `DOCUMENTO`) e na `UnB` (`PASTA`, `AGUARDE`).
const ARGUMENTOS: usize = 16;
const ARGUMENTOS_DA_PASTA: usize = 15;

/// Um documento lido, com o pai e, se for de pasta, a identidade da barra.
pub(super) struct DocumentoLido {
    pai: Pai,
    documento: DocumentoNaArvore,
    barra: Barra,
}

/// Lê os nós da árvore e confere a identidade: o NUP do nó `PROCESSO` tem
/// de ser **igual** ao pedido (comparação opaca) e o id igual ao aberto.
/// As pastas carregadas são conferidas pela identidade dos documentos.
pub(crate) fn ler_arvore(
    html: &str,
    nup_pedido: &str,
    id_aberto: &str,
) -> Result<ArvoreLida, ErroArvore> {
    ler_arvore_com_modo(html, nup_pedido, id_aberto, false)
}

#[cfg(test)]
pub(crate) fn ler_arvore_parcial(
    html: &str,
    nup_pedido: &str,
    id_aberto: &str,
) -> Result<ArvoreLida, ErroArvore> {
    ler_arvore_com_modo(html, nup_pedido, id_aberto, true)
}

/// Um documento movido pertence à lista documental, que pode ser não confiável.
fn filtrar_movidos(incluidos: bool, movidos: Vec<DocumentoMovido>) -> Vec<DocumentoMovido> {
    if incluidos { movidos } else { Vec::new() }
}

pub(crate) fn ler_arvore_com_modo(
    html: &str,
    nup_pedido: &str,
    id_aberto: &str,
    permitir_parcial: bool,
) -> Result<ArvoreLida, ErroArvore> {
    let mut processo: Option<ProcessoNaArvore> = None;
    let mut anexados: Vec<AnexadoLido> = Vec::new();
    let mut movidos: Vec<MovidoLido> = Vec::new();
    let mut documentos: Vec<DocumentoLido> = Vec::new();
    // Índice do nó → posição em `documentos`, só para documentos de pasta.
    let mut de_pasta: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut nos_de_pasta: Vec<NoDePasta> = Vec::new();
    let mut protocolos: Vec<(usize, String)> = Vec::new();
    let mut proximo = 0usize;
    for linha in html.split('\n') {
        if let Some(declarados) = ler_protocolos(linha)? {
            protocolos.push(declarados);
            continue;
        }
        // A concatenação vazia não muda a barra já lida do documento.
        if let Some(i) = literais::concatenacao_vazia_da_barra(linha) {
            if let Some(&posicao) = de_pasta.get(&i)
                && documentos[posicao].barra == Barra::NaoLida
            {
                return Err(ErroHtml::valor("barra_de_no_fora_da_forma_observada").into());
            }
            continue;
        }
        if let Some(i) = literais::indice_da_barra(linha)
            && let Some(&posicao) = de_pasta.get(&i)
        {
            let Some((_, barra)) = ler_barra(linha)? else {
                return Err(ErroHtml::valor("barra_de_no_fora_da_forma_observada").into());
            };
            documentos[posicao].barra = pastas::identidade_da_barra(&barra)?
                .map_or(Barra::SemIdentidade, Barra::Identidade);
            continue;
        }
        let Some((indice, argumentos)) = ler_declaracao(linha)? else {
            continue;
        };
        if indice != proximo {
            return Err(ErroHtml::valor("indice_de_no_fora_da_sequencia").into());
        }
        proximo += 1;
        let tipo = argumentos
            .first()
            .ok_or(ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada)?;
        match texto(tipo)? {
            "PROCESSO" => {
                if processo.is_some() {
                    return Err(ErroHtml::repetido("nos_do_processo_repetidos_na_arvore").into());
                }
                processo = Some(ler_processo(&com_aridade(argumentos)?)?);
            }
            "PROCESSO_ANEXADO" => anexados.push(ler_anexado(&com_aridade(argumentos)?)?),
            "DOCUMENTO_MOVIDO" => movidos.push(ler_movido(&com_aridade(argumentos)?)?),
            "DOCUMENTO" => {
                let lido = ler_documento(&com_aridade(argumentos)?)?;
                if documentos
                    .iter()
                    .any(|d| d.documento.id == lido.documento.id)
                {
                    return Err(ErroHtml::repetido("documento_repetido_na_arvore").into());
                }
                if matches!(lido.pai, Pai::Pasta(_)) {
                    de_pasta.insert(indice, documentos.len());
                }
                documentos.push(lido);
            }
            "PASTA" | "AGUARDE" => {
                let argumentos: [Literal; ARGUMENTOS_DA_PASTA] = argumentos
                    .try_into()
                    .map_err(|_| ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada)?;
                nos_de_pasta.push(pastas::ler_no_de_pasta(&argumentos)?);
            }
            _ => return Err(ErroArvore::TipoDeNoDaArvoreNaoSuportado),
        }
    }
    let processo = processo.ok_or(ErroArvore::NoDoProcessoAusenteNaArvore)?;
    if processo.nup != nup_pedido {
        return Err(ErroArvore::IdentidadeDoProcessoDiferenteDaPedida);
    }
    if processo.id != id_aberto {
        return Err(ErroArvore::IdDoProcessoNaArvoreDiferenteDoAberto);
    }
    let anexados = conferir_anexados(&processo, anexados, &documentos, &nos_de_pasta)?;
    let documentos_movidos = conferir_movidos(&processo, movidos, &documentos)?;
    let conferencia = conferencia_parcial::conferir(
        &processo.id,
        &nos_de_pasta,
        &protocolos,
        documentos,
        permitir_parcial,
    )?;
    Ok(ArvoreLida {
        processo,
        documentos: conferencia.documentos,
        pastas: conferencia.pastas,
        pastas_nao_carregadas: conferencia.pastas_nao_carregadas,
        documentos_incluidos: conferencia.documentos_incluidos,
        aviso_leitura_parcial: conferencia.aviso_leitura_parcial,
        anexados,
        documentos_movidos: filtrar_movidos(conferencia.documentos_incluidos, documentos_movidos),
        anexado_a: None,
        aberto_em: situacao::ler_aberto_em(html),
    })
}

fn com_aridade(argumentos: Vec<Literal>) -> Result<[Literal; ARGUMENTOS], ErroArvore> {
    argumentos
        .try_into()
        .map_err(|_| ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada)
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

fn decimal(valor: &str) -> bool {
    !valor.is_empty() && valor.bytes().all(|b| b.is_ascii_digit())
}

fn id(literal: &Literal) -> Result<String, ErroArvore> {
    let id = texto(literal)?;
    if !decimal(id) {
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

/// O pai (a2: o processo ou `PASTA<k>`) e o documento; a dica (a6) só
/// conta para o cancelamento.
fn ler_documento(a: &[Literal; ARGUMENTOS]) -> Result<DocumentoLido, ErroArvore> {
    let id_documento = id(&a[1])?;
    let pai = pastas::pai_do_documento(texto(&a[2])?)?;
    let pasta = match &pai {
        Pai::Pasta(k) => Some(format!("PASTA{k}")),
        Pai::Processo(_) => None,
    };
    Ok(DocumentoLido {
        pai,
        documento: DocumentoNaArvore {
            id: id_documento,
            rotulo: texto(&a[5])?.to_owned(),
            numero_sei: texto_ou_nulo(&a[15])?.map(str::to_owned),
            pasta,
            natureza: natureza::ler_natureza(a),
        },
        barra: Barra::NaoLida,
    })
}

/// O NUP (a5) do nó `PROCESSO` declarado, sem conferência: o replay do
/// acervo o usa para tirar da gravação a identidade pedida, e a leitura de
/// documento (Refs #60) para conferir a árvore aberta por número SEI, que
/// não conhece o NUP antes dela; [`ler_arvore`] confere o resto.
pub(crate) fn nup_declarado_na_arvore(html: &str) -> Option<String> {
    html.split('\n').find_map(|linha| {
        let (_, argumentos) = ler_declaracao(linha).ok()??;
        match argumentos.as_slice() {
            [Literal::Texto(tipo), _, _, _, _, Literal::Texto(nup), ..]
                if tipo == "PROCESSO" && argumentos.len() == ARGUMENTOS =>
            {
                Some(nup.clone())
            }
            _ => None,
        }
    })
}
