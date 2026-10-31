//! Busca textual restrita aos corpos de documentos observados em cassette verificada.
use crate::cassette;
use crate::documento_web::{
    ErroCorpoDocumento, ErroTituloDocumento, IdentidadeDoDocumento, identidade_do_titulo,
    texto_do_corpo,
};
use serde_json::json;
use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroBuscaDocumento {
    TermoVazio,
    TermoComControle,
    TermoAcimaDoLimite,
    FonteNaoVerificada,
    NumeroDocumentoRepetido,
    Titulo(ErroTituloDocumento),
    Corpo(ErroCorpoDocumento),
}
impl ErroBuscaDocumento {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::TermoVazio => "termo_de_documento_vazio",
            Self::TermoComControle => "termo_de_documento_contem_controle",
            Self::TermoAcimaDoLimite => "termo_de_documento_acima_do_limite",
            Self::FonteNaoVerificada => "documentos_de_busca_nao_verificados",
            Self::NumeroDocumentoRepetido => "numero_documento_repetido_na_busca",
            Self::Titulo(erro) => erro.subcodigo(),
            Self::Corpo(erro) => erro.subcodigo(),
        }
    }
    fn codigo(self) -> (&'static str, u8) {
        if matches!(
            self,
            Self::TermoVazio | Self::TermoComControle | Self::TermoAcimaDoLimite
        ) {
            ("entrada_invalida", 2)
        } else {
            ("fonte_invalida", 1)
        }
    }
}
fn validar_termo(termo: &str) -> Result<&str, ErroBuscaDocumento> {
    let termo = termo.trim();
    if termo.is_empty() {
        return Err(ErroBuscaDocumento::TermoVazio);
    }
    if termo.chars().any(char::is_control) {
        return Err(ErroBuscaDocumento::TermoComControle);
    }
    if termo.chars().count() > 100 {
        return Err(ErroBuscaDocumento::TermoAcimaDoLimite);
    }
    Ok(termo)
}
fn buscar(htmls: &[String], termo: &str) -> Result<Vec<IdentidadeDoDocumento>, ErroBuscaDocumento> {
    let procurado = validar_termo(termo)?.to_lowercase();
    if htmls.is_empty() {
        return Err(ErroBuscaDocumento::FonteNaoVerificada);
    }
    let mut observados = HashSet::new();
    let mut resultado = Vec::new();
    for html in htmls {
        let identidade = identidade_do_titulo(html).map_err(ErroBuscaDocumento::Titulo)?;
        if !observados.insert(identidade.numero.clone()) {
            return Err(ErroBuscaDocumento::NumeroDocumentoRepetido);
        }
        let corpo = texto_do_corpo(html).map_err(ErroBuscaDocumento::Corpo)?;
        if corpo.to_lowercase().contains(&procurado) {
            resultado.push(identidade);
        }
    }
    Ok(resultado)
}
pub(super) fn mostrar(
    card: &Path,
    arquivo: &Path,
    passos: &[usize],
    termo: &str,
    json_mode: bool,
) -> ExitCode {
    let resultado = validar_termo(termo).and_then(|_| {
        cassette::read_cassette_documents(card, arquivo, passos)
            .map_err(|_| ErroBuscaDocumento::FonteNaoVerificada)
            .and_then(|(htmls, instalacao)| {
                buscar(&htmls, termo).map(|matches| (matches, instalacao))
            })
    });
    match resultado {
        Ok((documentos, instalacao)) => {
            if json_mode {
                println!(
                    "{}",
                    json!({"documentos":documentos,"instalacao":instalacao,
                   "escopo":"corpos_documentos_capturados_offline","total_encontrado":documentos.len()})
                );
            } else {
                println!(
                    "IES: {} documento(s) observados na busca offline.",
                    documentos.len()
                );
                for doc in documentos {
                    println!("Documento SEI {}: {}", doc.numero, doc.tipo);
                }
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            let (codigo, saida) = erro.codigo();
            if json_mode {
                println!(
                    "{}",
                    json!({"codigo":codigo,"subcodigo":erro.subcodigo(),"retentavel":false})
                );
            } else {
                eprintln!("IES: busca offline não concluída [{}]", erro.subcodigo());
            }
            ExitCode::from(saida)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn busca_por_termo_preserva_identidades_e_nao_publica_conteudo() {
        let paginas=vec![
            "<html><title>SEI/UnB - 0007 - Ofício</title><body><p>Designação de assessor</p></body></html>".to_owned(),
            "<html><title>SEI/UnB - 0008 - Memorando</title><body><p>Outro assunto</p></body></html>".to_owned()
        ];
        let achados = buscar(&paginas, "ASSESSOR").unwrap();
        assert_eq!(achados.len(), 1);
        assert_eq!(achados[0].numero, "0007");
    }
    #[test]
    fn ausencia_em_cassette_eh_lista_vazia_nao_inexistencia_live() {
        let paginas = vec!["<title>SEI/UnB - 0007 - Ofício</title><body>Juntada</body>".to_owned()];
        assert_eq!(
            buscar(&paginas, "inexistente").unwrap(),
            Vec::<IdentidadeDoDocumento>::new()
        );
    }
    #[test]
    fn falhas_sao_variantes_tipadas_especificas() {
        assert_eq!(buscar(&[], " "), Err(ErroBuscaDocumento::TermoVazio));
        assert_eq!(
            buscar(&[], "linha\nquebrada"),
            Err(ErroBuscaDocumento::TermoComControle)
        );
        let paginas = vec![
            "<title>SEI/UnB - 0007 - Ofício</title><body><script>segredo</script></body>"
                .to_owned(),
        ];
        assert_eq!(
            buscar(&paginas, "segredo"),
            Err(ErroBuscaDocumento::Corpo(
                ErroCorpoDocumento::CorpoComConteudoExecutavel
            ))
        );
    }
    #[test]
    fn titulo_repetido_nao_fabrica_resultados_distintos() {
        let html = "<title>SEI/UnB - 0007 - Ofício</title><body>busca</body>".to_owned();
        assert_eq!(
            buscar(&[html.clone(), html], "busca"),
            Err(ErroBuscaDocumento::NumeroDocumentoRepetido)
        );
    }
}
