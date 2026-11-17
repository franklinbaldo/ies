//! Leitura estritamente offline do histórico Web de um processo.
//! Não confundir histórico parcial com a representação integral do processo.
use crate::processo::{Andamento, DetalheProcesso, NumeroProcesso};

/// Condições específicas do parser, independentes de mensagens.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroHistorico {
    NumeroDoHistoricoDiferenteDoSolicitado,
    HistoricoVazio,
    HistoricoAusente,
    HistoricoInvalido,
    LinhaInvalida,
    NumeroDoHistoricoInvalido(#[source] crate::processo::NumeroProcessoInvalido),
    TabelasDoHistoricoRepetidas,
    TituloAusente,
    TituloInvalido,
    TitulosDoHistoricoRepetidos,
}

fn texto(no: crate::tela_html::Elemento<'_>) -> String {
    let valor = no.texto();
    valor
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// O título do SVG não pode assumir a identidade do processo: só vale o
/// `title` do `head`. Como no navegador, o documento tem um único `head`.
fn titulo_do_historico(dom: &crate::tela_html::Documento) -> Result<String, ErroHistorico> {
    let mut titulos = dom
        .elementos("head")
        .flat_map(|cabecalho| cabecalho.elementos("title"));
    let primeiro = titulos.next().ok_or(ErroHistorico::TituloAusente)?;
    if titulos.next().is_some() {
        return Err(ErroHistorico::TitulosDoHistoricoRepetidos);
    }
    Ok(texto(primeiro))
}

#[cfg(test)]
pub fn numero_do_historico(html: &str) -> Result<String, ErroHistorico> {
    let servido = crate::tela_html::HtmlServido::de_teste(html);
    let dom = crate::tela_html::arvore(&servido);
    let titulo = titulo_do_historico(&dom)?;
    let (_, protocolo) = titulo
        .split_once("Histórico do Processo ")
        .ok_or(ErroHistorico::TituloInvalido)?;
    let numero =
        NumeroProcesso::novo(protocolo).map_err(ErroHistorico::NumeroDoHistoricoInvalido)?;
    Ok(numero.as_str().to_owned())
}

pub fn ler_historico(
    html: &crate::tela_html::HtmlServido,
    numero: &str,
) -> Result<DetalheProcesso, ErroHistorico> {
    let numero = NumeroProcesso::novo(numero).map_err(ErroHistorico::NumeroDoHistoricoInvalido)?;
    let dom = crate::tela_html::arvore(html);
    let titulo = titulo_do_historico(&dom)?;
    // O protocolo é opaco. Não fragmentar por pontuação nem remover zeros.
    let (_, mostrado) = titulo
        .split_once("Histórico do Processo ")
        .ok_or(ErroHistorico::TituloInvalido)?;
    if mostrado != numero.as_str() {
        return Err(ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado);
    }
    // O primeiro ID não é suficiente: tabelas duplicadas podem ocultar andamentos.
    let candidatas = dom.com_id("tblHistorico").collect::<Vec<_>>();
    let primeira = match candidatas.as_slice() {
        [] => return Err(ErroHistorico::HistoricoAusente),
        [primeira] => *primeira,
        _ => return Err(ErroHistorico::TabelasDoHistoricoRepetidas),
    };
    if primeira.nome() != "table" || primeira.elementos("table").next().is_some() {
        return Err(ErroHistorico::HistoricoInvalido);
    }
    let mut detalhe = DetalheProcesso::novo(numero);
    let linhas = primeira.elementos("tr");
    for linha in linhas {
        let cells: Vec<String> = linha.elementos("td").map(texto).collect();
        if cells.is_empty() {
            continue;
        }
        let Ok([ocorrido_em, unidade, usuario, descricao]) = <[String; 4]>::try_from(cells) else {
            return Err(ErroHistorico::LinhaInvalida);
        };
        if ocorrido_em.is_empty() || descricao.is_empty() {
            return Err(ErroHistorico::LinhaInvalida);
        }
        detalhe.andamentos.push(Andamento {
            ocorrido_em,
            unidade: (!unidade.is_empty()).then_some(unidade),
            usuario: (!usuario.is_empty()).then_some(usuario),
            descricao,
        });
    }
    if detalhe.andamentos.is_empty() {
        return Err(ErroHistorico::HistoricoVazio);
    }
    Ok(detalhe)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(numero: &str) -> String {
        format!(
            "<title>SEI - Histórico do Processo {numero}&nbsp;</title>             <table id='tblHistorico'><tr><th>Data/Hora</th></tr>             <tr class='andamentoAberto'><td>12/07/2026 17:49</td>             <td>UNIDADE A</td><td>usuario</td>             <td>Processo recebido na unidade</td></tr></table>"
        )
    }

    #[test]
    fn interpreta_historico_sem_inventar_identificadores() {
        let resultado = ler_historico(
            &crate::tela_html::HtmlServido::de_teste(&fixture("202600005002487")),
            "202600005002487",
        )
        .unwrap();
        assert_eq!(resultado.andamentos.len(), 1);
        assert_eq!(resultado.andamentos[0].usuario.as_deref(), Some("usuario"));
        assert!(resultado.id_interno.is_none());
    }

    #[test]
    fn booleano_antes_do_id_nao_esconde_a_tabela_do_historico() {
        let html =
            crate::tela_html::teste::com_booleano_antes_dos_atributos(&fixture("202600005002487"));
        let resultado = ler_historico(
            &crate::tela_html::HtmlServido::de_teste(&html),
            "202600005002487",
        )
        .unwrap();
        assert_eq!(resultado.andamentos.len(), 1);
    }

    #[test]
    fn historico_aceita_protocolo_opaco_sem_mascara() {
        for protocolo in [
            "23106.000001/2026-01",
            "550e8400-e29b-41d4-a716-446655440000",
            "000123",
            "AB-C",
        ] {
            let html = fixture(protocolo);
            assert_eq!(numero_do_historico(&html).unwrap(), protocolo);
            assert_eq!(
                ler_historico(&crate::tela_html::HtmlServido::de_teste(&html), protocolo)
                    .unwrap()
                    .numero
                    .as_str(),
                protocolo
            );
        }
    }

    #[test]
    fn historico_nao_confunde_prefixos_de_protocolo() {
        let html = crate::tela_html::HtmlServido::de_teste(&fixture("AB-C"));
        assert_eq!(
            ler_historico(&html, "ABC").unwrap_err(),
            ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado
        );
        assert_eq!(
            ler_historico(
                &crate::tela_html::HtmlServido::de_teste(&fixture("000123")),
                "123"
            )
            .unwrap_err(),
            ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado
        );
    }

    #[test]
    fn recusa_outro_processo_e_tabela_incompleta() {
        assert_eq!(
            ler_historico(
                &crate::tela_html::HtmlServido::de_teste(&fixture("202600005002487")),
                "202600005002488"
            )
            .unwrap_err(),
            ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado
        );
        let invalido = crate::tela_html::HtmlServido::de_teste(
            "<title>Histórico do Processo 202600005002487</title>        <table id='tblHistorico'><tr><td>12/07/2026</td></tr></table>",
        );
        assert_eq!(
            ler_historico(&invalido, "202600005002487").unwrap_err(),
            ErroHistorico::LinhaInvalida
        );
    }

    #[test]
    fn duas_tabelas_de_historico_falham_sem_omitir_andamentos() {
        let numero = "000123";
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "{}<table id='tblHistorico'><tr><td>13/07/2026</td><td>U</td><td>X</td><td>Outro andamento</td></tr></table>",
            fixture(numero)
        ));
        let erro = ler_historico(&html, numero).unwrap_err();
        assert_eq!(erro, ErroHistorico::TabelasDoHistoricoRepetidas);
    }

    #[test]
    fn tabela_com_id_de_historico_em_div_e_table_e_ambigua() {
        let numero = "000123";
        // A `div` vem depois do título: antes dele, o navegador já abre o
        // `body` e o `title` deixa de ser o do `head`.
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "{}<div id='tblHistorico'>navegação</div>",
            fixture(numero)
        ));
        assert_eq!(
            ler_historico(&html, numero).unwrap_err(),
            ErroHistorico::TabelasDoHistoricoRepetidas
        );
    }

    #[test]
    fn svg_no_body_nao_substitui_identidade_do_processo() {
        let html = "<html><head><title>SEI - Histórico do Processo 000123</title></head>
            <body><svg><title>Histórico do Processo INTRUSO</title></svg>
            <table id='tblHistorico'>
              <tr><td>12/07/2026</td><td>U</td><td>X</td><td>Movimento</td></tr>
            </table></body></html>";
        assert_eq!(numero_do_historico(html).unwrap(), "000123");
        assert_eq!(
            ler_historico(&crate::tela_html::HtmlServido::de_teste(html), "000123")
                .unwrap()
                .andamentos
                .len(),
            1
        );
    }

    #[test]
    fn svg_nao_corrige_head_sem_titulo() {
        let html = "<html><head></head><body><svg>
            <title>SEI - Histórico do Processo 000123</title>
            </svg></body></html>";
        assert_eq!(numero_do_historico(html), Err(ErroHistorico::TituloAusente));
        assert_eq!(
            ler_historico(&crate::tela_html::HtmlServido::de_teste(html), "000123"),
            Err(ErroHistorico::TituloAusente)
        );
    }

    #[test]
    fn recusa_titulos_repetidos_mesmo_em_dois_head() {
        let titulos = "<html><head>
            <title>SEI - Histórico do Processo 000123</title>
            <title>SEI - Histórico do Processo 000124</title></head></html>";
        assert_eq!(
            numero_do_historico(titulos),
            Err(ErroHistorico::TitulosDoHistoricoRepetidos)
        );
        assert_eq!(
            ler_historico(&crate::tela_html::HtmlServido::de_teste(titulos), "000123"),
            Err(ErroHistorico::TitulosDoHistoricoRepetidos)
        );
        // O navegador ignora o segundo `<head>`: os dois `title` caem no
        // único `head` do documento.
        let cabecalhos = "<html>
            <head><title>SEI - Histórico do Processo 000123</title></head>
            <head><title>SEI - Histórico do Processo 000124</title></head></html>";
        assert_eq!(
            numero_do_historico(cabecalhos),
            Err(ErroHistorico::TitulosDoHistoricoRepetidos)
        );
        assert_eq!(
            ler_historico(
                &crate::tela_html::HtmlServido::de_teste(cabecalhos),
                "000123"
            ),
            Err(ErroHistorico::TitulosDoHistoricoRepetidos)
        );
    }
}
