//! Saída do documento externo em `ies documento DOC` (Refs #60, fatia 3;
//! contracts/cli/documento-obter-externo.md): o objeto `anexo` do JSON,
//! a linha humana do anexo e a recusa do limite inválido. Bytes do
//! arquivo nunca vão para a saída padrão.
use crate::anexo_documento::{AnexoLido, VARIAVEL_DO_LIMITE};
use serde_json::{Value, json};
use std::process::ExitCode;

/// `mime`, `tamanho`, `sha256`, `nome_original` e `disposicao`.
pub(super) fn json_do_anexo(a: &AnexoLido) -> Value {
    json!({
        "mime": a.mime,
        "tamanho": a.tamanho,
        "sha256": a.sha256,
        "nome_original": a.nome_original,
        "disposicao": a.disposicao.map(crate::anexo_documento::Disposicao::como_texto),
    })
}

/// A linha humana do anexo; sem texto nem arquivo, a dica de `--saida`.
pub(super) fn linha_do_anexo(a: &AnexoLido, gravado: bool) -> String {
    let nome = a
        .nome_original
        .as_deref()
        .map_or_else(String::new, |n| format!(", nome original {n:?}"));
    let dica = if gravado || a.texto.is_some() {
        ""
    } else {
        " Use --saida DIR para gravar o arquivo."
    };
    format!(
        "Anexo {}: {} bytes, sha256 {}{nome}.{dica}",
        a.mime, a.tamanho, a.sha256
    )
}

/// `IES_LIMITE_ANEXO_BYTES` inválido: entrada inválida, saída 2.
pub(super) fn limite_invalido(json_mode: bool) -> ExitCode {
    let mensagem = format!("{VARIAVEL_DO_LIMITE} tem de ser um inteiro de 1 a 2147483648");
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anexo_documento::Disposicao;

    fn anexo(texto: Option<&str>) -> AnexoLido {
        AnexoLido {
            mime: "application/pdf".to_owned(),
            tamanho: 12,
            sha256: "ab".repeat(32),
            nome_original: Some("Minuta.pdf".to_owned()),
            disposicao: Some(Disposicao::Inline),
            texto: texto.map(str::to_owned),
            arquivo: None,
        }
    }

    #[test]
    fn json_e_linha_do_anexo_sem_bytes_do_arquivo() {
        assert_eq!(
            json_do_anexo(&anexo(None)),
            json!({"mime": "application/pdf", "tamanho": 12, "sha256": "ab".repeat(32),
                   "nome_original": "Minuta.pdf", "disposicao": "inline"})
        );
        let linha = linha_do_anexo(&anexo(None), false);
        assert!(
            linha.ends_with("Use --saida DIR para gravar o arquivo."),
            "{linha}"
        );
        assert!(linha.contains("12 bytes"));
        assert!(!linha_do_anexo(&anexo(Some("t")), false).contains("--saida"));
        assert!(!linha_do_anexo(&anexo(None), true).contains("--saida"));
    }
}
