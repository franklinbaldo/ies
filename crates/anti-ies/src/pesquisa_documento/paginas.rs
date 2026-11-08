//! Projeção HTML da pesquisa paginada: documento/versionamento do mesmo SQLite.
//! Não conserva páginas gravadas, cookies, URLs assinadas nem HTML das cassettes.
use crate::{Estado, escapar};
use iesei::documento_versoes;
use iesei::instalacao::Instalacao;
use iesei::pesquisa_documento::PaginaObservada;
use rusqlite::Connection;
use std::fmt::Write as _;

const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

fn milhar(numero: u64) -> String {
    let texto = numero.to_string();
    let mut formatado = String::new();
    for (indice, digito) in texto.chars().enumerate() {
        if indice > 0 && (texto.len() - indice).is_multiple_of(3) {
            formatado.push('.');
        }
        formatado.push(digito);
    }
    formatado
}

/// Cada link vem da identidade documental já registrada por outra operação.
/// O simulador não fabrica uma linha para um ID sem documento conhecido.
pub(super) fn renderizar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    pagina: &PaginaObservada,
    tamanho: u64,
) -> Result<String, Estado> {
    if tamanho == 0
        || pagina.inicio != (u64::from(pagina.pagina) - 1) * tamanho + 1
        || pagina.fim > (u64::from(pagina.pagina) * tamanho).min(pagina.total_declarado)
    {
        return Err(Estado::DadosInsuficientes);
    }
    let mut linhas = String::new();
    for linha in &pagina.linhas {
        let documento = documento_versoes::ler(banco, instalacao, cenario, &linha.id_documento)
            .map_err(|_| Estado::DadosInsuficientes)?
            .ok_or(Estado::DadosInsuficientes)?;
        let url_processo = escapar(&format!(
            "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={}&id_documento={}&{CONTEXTO}&infra_hash={}",
            documento.id_procedimento,
            documento.id_documento,
            crate::protocolo_sintetico::INFRA_HASH
        ));
        let url_documento = escapar(&match linha.id_anexo.as_deref() {
            Some(id) => format!(
                "controlador.php?acao=documento_download_anexo&acao_origem=protocolo_pesquisar&id_anexo={id}&{CONTEXTO}&infra_hash={}",
                crate::protocolo_sintetico::INFRA_HASH
            ),
            None => format!(
                "controlador.php?acao=documento_visualizar&acao_origem=protocolo_pesquisar&id_documento={}&{CONTEXTO}&infra_hash={}",
                documento.id_documento,
                crate::protocolo_sintetico::INFRA_HASH
            ),
        });
        write!(
            linhas,
            "<tr class='pesquisaTituloRegistro'><td colspan='2' class='pesquisaTituloEsquerda'>\
             <a class='arvore' href='{url_processo}'><img class='arvore' src='svg/arvore.svg' alt='Documento'/></a>\
             <a class='protocoloNormal' href='{url_processo}'>{}</a> (\
             <a class='protocoloNormal' href='{url_documento}'>{}</a>)</td>\
             <td class='pesquisaTituloDireita'><a class='protocoloNormal' href='{url_documento}'>{}</a></td></tr>\
             <tr><td colspan='3' class='pesquisaSnippet'>Documento observado</td></tr><tr>",
            escapar(&documento.numero_processo),
            escapar(&linha.rotulo),
            escapar(&documento.numero)
        )
        .map_err(|_| Estado::DadosInsuficientes)?;
        if let Some(unidade) = &linha.unidade {
            write!(
                linhas,
                "<td class='pesquisaMetatag'><b>Unidade:</b> <a class='ancoraSigla'>{}</a></td>",
                escapar(unidade)
            )
            .map_err(|_| Estado::DadosInsuficientes)?;
        }
        linhas.push_str("<td class='pesquisaMetatag'><b>Usuário:</b></td>");
        if let Some(inclusao) = &linha.inclusao {
            write!(
                linhas,
                "<td class='pesquisaMetatag'><b>Inclusão:</b> {}</td>",
                escapar(inclusao)
            )
            .map_err(|_| Estado::DadosInsuficientes)?;
        }
        linhas.push_str("</tr>");
    }

    let mut links = String::new();
    for indice in 0..10u64 {
        let deslocamento = indice * tamanho;
        if deslocamento < pagina.total_declarado && indice + 1 != u64::from(pagina.pagina) {
            write!(
                links,
                "<a href=\"javascript:navegar('{deslocamento}')\">{}</a> ",
                indice + 1
            )
            .map_err(|_| Estado::DadosInsuficientes)?;
        }
    }
    Ok(format!(
        "<div class='pesquisaBarra'><div class='pesquisaBarraE'></div>\
         <div class='pesquisaBarraD'>Exibindo {} - {} de {}</div></div>\
         <table class='pesquisaResultado'>{linhas}</table>\
         <div class='pesquisaPaginas'><div class='pesquisaPaginaSelecionada'>{}</div>{links}</div>",
        milhar(pagina.inicio),
        milhar(pagina.fim),
        milhar(pagina.total_declarado),
        pagina.pagina
    ))
}
