//! Projeção tipada de processos nas respostas HTML de pesquisa do SEI Web.
//! Parser puro: não executa links nem presume permissão/resultado live.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroConsultaDoControlador, menciona_acao,
};
use crate::processo::{NumeroProcesso, NumeroProcessoInvalido};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultadoPesquisa {
    pub numero: NumeroProcesso,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroResultadoPesquisa {
    LinhaDePesquisaSemLinkDoProcesso,
    LinkDeProcessoSemClasseReconhecida,
    /// `a.arvore` da linha com `href` diferente do `a.protocoloNormal`.
    LinkDaArvoreComHrefDiferenteDoLinkDoProcesso,
    LinkDePesquisaNaoAbreProcesso,
    LinkDePesquisaSemIdentificadorDoProcesso,
    /// A consulta do link de pesquisa não se deixa ler em pares.
    ConsultaDoLinkIlegivel(#[source] ErroConsultaDoControlador),
    /// A `acao` do link de pesquisa não é única nem conhecida.
    AcaoDoLinkNaoLida(#[source] crate::consulta_do_controlador::ErroAcaoDaUrl),
    LinkDePesquisaComIdentificadorDoProcessoNaoDecimal,
    NumeroDePesquisa(#[from] NumeroProcessoInvalido),
    /// `a.protocoloNormal` de `documento_visualizar` na linha cujo link do
    /// processo não leva o mesmo `id_documento` (Goiás, acerto em documento).
    LinkDoDocumentoDaLinhaDiferenteDoProcesso,
    /// `href` de um `a.protocoloNormal` da linha ilegível.
    IdentificadorDoProcessoRepetidoNaPesquisa,
    LinhaDePesquisaComMaisDeUmLinkDaArvore,
    LinhaDePesquisaComMaisDeUmLinkDoProcesso,
    LinkDePesquisaComIdentificadoresRepetidos,
    LinkDeProcessoForaDaLinhaDePesquisa,
    NumeroDePesquisaRepetido,
    TabelaDePesquisaAusente,
    TabelasDePesquisaRepetidas,
}

/// Confere a ação e a identidade de um href Web sem realizar a navegação.
/// `href` chega com as entidades resolvidas (`valor_decodificado`).
fn validar_href_de_processo(href: &str) -> Result<&str, ErroResultadoPesquisa> {
    // Não aceitar múltiplos parâmetros de ação: proxies/servidores podem
    // escolher o primeiro ou o último, tornando o destino indeterminado.
    let query = href
        .strip_prefix("controlador.php?")
        .ok_or(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)?;
    let pares =
        ConsultaDoControlador::ler(query).map_err(ErroResultadoPesquisa::ConsultaDoLinkIlegivel)?;
    let acao = pares
        .acao()
        .map_err(ErroResultadoPesquisa::AcaoDoLinkNaoLida)?;
    if acao != AcaoDoSei::ProcedimentoTrabalhar.como_na_url() {
        return Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso);
    }
    // Um link de trabalho sem ID interno não endereça um processo.
    // Dois IDs tornam o destino dependente da interpretação do servidor.
    // O identificador é validado somente para uso local e não é serializado.
    match pares
        .valores("id_procedimento")
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | [""] => Err(ErroResultadoPesquisa::LinkDePesquisaSemIdentificadorDoProcesso),
        [id] if id.bytes().all(|byte| byte.is_ascii_digit()) => Ok(*id),
        [_] => Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorDoProcessoNaoDecimal),
        _ => Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadoresRepetidos),
    }
}

/// A `UnB` 4.0.9 publica, antes do número, um `a.arvore` (ícone da árvore)
/// com o **mesmo** `href` do `a.protocoloNormal`. Só essa forma é aceita:
/// zero ou um por linha, `href` byte a byte igual.
fn contar_arvore_da_linha(
    linha: crate::tela_html::Elemento<'_>,
    href_do_processo: &str,
) -> Result<usize, ErroResultadoPesquisa> {
    // O `a.protocoloNormal` da mesma linha repete `class`: leitura tolerante.
    let arvores = linha
        .elementos("a")
        .filter(|elemento| elemento.tem_classe("arvore"))
        .collect::<Vec<_>>();
    let arvore = match arvores.as_slice() {
        [] => return Ok(0),
        [arvore] => *arvore,
        _ => {
            return Err(ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDaArvore);
        }
    };
    if arvore.atributo("href") != Some(href_do_processo) {
        return Err(ErroResultadoPesquisa::LinkDaArvoreComHrefDiferenteDoLinkDoProcesso);
    }
    Ok(1)
}

/// Resultados observados somente na tabela de pesquisa retornada pelo SEI.
/// Nunca serializa links assinados, IDs técnicos, trechos HTML ou filtros.
pub fn ler_resultados(
    html: &crate::tela_html::HtmlServido,
) -> Result<Vec<ResultadoPesquisa>, ErroResultadoPesquisa> {
    let dom = crate::tela_html::arvore(html);
    let primeira = match dom
        .elementos("table")
        .filter(|elemento| elemento.tem_classe("pesquisaResultado"))
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => return Err(ErroResultadoPesquisa::TabelaDePesquisaAusente),
        [primeira] => *primeira,
        _ => return Err(ErroResultadoPesquisa::TabelasDePesquisaRepetidas),
    };
    let linhas = primeira
        .elementos("tr")
        .filter(|elemento| elemento.tem_classe("pesquisaTituloRegistro"))
        .collect::<Vec<_>>();
    let mut observados = HashSet::new();
    let mut ids_observados = HashSet::new();
    let mut resultados = Vec::new();
    let mut arvores = 0;
    let mut links_de_documento = 0;
    for linha in linhas {
        let links = linha
            .elementos("a")
            .filter(|elemento| elemento.tem_classe("protocoloNormal"))
            .collect::<Vec<_>>();
        // Links que só abrem um documento (acerto em documento, Goiás) não
        // disputam o lugar do link do processo.
        let (candidatos, documentos) = documento_da_linha::separar(links);
        let link_no = match candidatos.as_slice() {
            [] => return Err(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso),
            [link] => *link,
            _ => {
                return Err(ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDoProcesso);
            }
        };
        let href = link_no
            .atributo("href")
            .map(str::to_owned)
            .ok_or(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)?;
        let primeiro_id = validar_href_de_processo(&href)?;
        if !documentos.is_empty() {
            let doc = documento_da_linha::documento_do_processo(&href);
            if documentos.iter().any(|d| Some(d.as_str()) != doc) {
                return Err(ErroResultadoPesquisa::LinkDoDocumentoDaLinhaDiferenteDoProcesso);
            }
        }
        links_de_documento += documentos.len();
        let numero = NumeroProcesso::novo(&link_no.texto())?;
        if !observados.insert(numero.as_str().to_owned()) {
            return Err(ErroResultadoPesquisa::NumeroDePesquisaRepetido);
        }
        // O identificador interno é decimal: 7 e 007 denotam a mesma chave.
        // Normalização por string evita overflow em IDs maiores que u64.
        if !ids_observados.insert(primeiro_id.trim_start_matches('0').to_owned()) {
            return Err(ErroResultadoPesquisa::IdentificadorDoProcessoRepetidoNaPesquisa);
        }
        arvores += contar_arvore_da_linha(linha, &href)?;
        resultados.push(ResultadoPesquisa { numero });
    }
    // A seleção por classe das linhas pode omitir links reais se a versão
    // do SEI mudar o HTML. Nunca devolver subconjunto silencioso como se
    // representasse todos os links observáveis da tabela de resultados.
    let links_da_tabela = primeira
        .elementos("a")
        .filter(|elemento| elemento.tem_classe("protocoloNormal"))
        .collect::<Vec<_>>()
        .len();
    if links_da_tabela != resultados.len() + links_de_documento {
        return Err(ErroResultadoPesquisa::LinkDeProcessoForaDaLinhaDePesquisa);
    }
    let links_de_trabalho = links_de_trabalho(primeira);
    if links_de_trabalho != resultados.len() + arvores {
        return Err(ErroResultadoPesquisa::LinkDeProcessoSemClasseReconhecida);
    }
    Ok(resultados)
}

/// Todos os links da tabela que abrem `procedimento_trabalhar`, com ou sem
/// a classe esperada.
fn links_de_trabalho(tabela: crate::tela_html::Elemento<'_>) -> usize {
    // A classe CSS do link não é garantia de que ele seja o único link
    // de trabalho na tabela. Uma versão diferente do SEI pode perder
    // essa classe e ocultar processos na projeção: falhar fechado.
    let mut links_de_trabalho = 0;
    for no in tabela.elementos("a") {
        let abre_processo = no
            .atributo("href")
            .map(str::to_owned)
            .is_some_and(|href| menciona_acao(&href, AcaoDoSei::ProcedimentoTrabalhar));
        links_de_trabalho += usize::from(abre_processo);
    }
    links_de_trabalho
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_arvore;

#[cfg(test)]
pub(crate) mod tests_fixtures;

// Linha com acerto em documento (Goiás): só o processo é publicado.
mod documento_da_linha;

// Replay de conformidade do acervo público pelas peças do comando (Refs #58).
#[cfg(test)]
mod acervo;

// A declaração do formulário fica com a pesquisa, não num catálogo do primitivo
// (docs/arquitetura/0002); `ies processo buscar` a usa pelo `fluxo`.
pub(crate) mod formulario_pesquisa;

// Pesquisa em documentos (`ies documento buscar`): linhas, 302 e resposta.
pub(crate) mod documentos;

// O 302 da pesquisa é interpretado pela própria operação, nunca seguido.
pub(crate) mod redirecionamento;

// Regras do `onsubmit` da pesquisa, lidas do JS servido e escritas à mão.
mod regras_formulario;

// Resposta 200 (lista com barra ou "sem resultado") e paginação `navegar()`.
pub(crate) mod paginacao;
pub(crate) mod resposta;

// Link do menu da pesquisa no Controle e o fluxo do comando.
pub(crate) mod alvo;
pub(crate) mod erro_busca;
pub(crate) mod fluxo;
pub(crate) mod link_da_pesquisa;

// Critérios opcionais derivados estritamente dos controles servidos.
pub(crate) mod filtros;
