//! #332 — inspeção local do PDF, inteiramente anterior à sessão e à rede.
//! O limite da instalação só será aplicado depois de observado no cadastro.

use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PdfLocal {
    pub(crate) nome: String,
    pub(crate) tamanho_bytes: u64,
    pub(crate) sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FalhaPdf {
    Ausente,
    NaoRegular,
    LinkSimbolico,
    Vazio,
    Extensao,
    Conteudo,
    AcimaDoLimite,
    Leitura,
}

impl FalhaPdf {
    pub(crate) const fn subcodigo(self) -> &'static str {
        match self {
            Self::Ausente => "arquivo_pdf_ausente",
            Self::NaoRegular => "arquivo_pdf_nao_regular",
            Self::LinkSimbolico => "arquivo_pdf_link_simbolico",
            Self::Vazio => "arquivo_pdf_vazio",
            Self::Extensao => "extensao_pdf_invalida",
            Self::Conteudo => "conteudo_nao_eh_pdf",
            Self::AcimaDoLimite => "arquivo_pdf_acima_do_limite",
            Self::Leitura => "leitura_do_arquivo_pdf_falhou",
        }
    }
}

/// Inspeção local; não prova que a instalação oferece tipo, limite ou upload.
/// Só aceita o caminho e a extensão fornecidos; não segue links simbólicos.
pub(crate) fn inspecionar_pdf(
    caminho: &Path,
    limite_observado: Option<u64>,
) -> Result<PdfLocal, FalhaPdf> {
    // A recusa acontece antes de ler qualquer arquivo de sessão ou enviar HTTP.
    for parte in caminho.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        let info = fs::symlink_metadata(parte).map_err(|erro| {
            if erro.kind() == ErrorKind::NotFound {
                FalhaPdf::Ausente
            } else {
                FalhaPdf::Leitura
            }
        })?;
        if info.file_type().is_symlink() {
            return Err(FalhaPdf::LinkSimbolico);
        }
    }
    if !fs::symlink_metadata(caminho)
        .map_err(|_| FalhaPdf::Leitura)?
        .is_file()
    {
        return Err(FalhaPdf::NaoRegular);
    }
    let nome = caminho
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(FalhaPdf::Extensao)?;
    if !caminho
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    {
        return Err(FalhaPdf::Extensao);
    }
    let mut abertura = OpenOptions::new();
    abertura.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // Mesmo O_NOFOLLOW/O_NONBLOCK da leitura privada do transporte.
        abertura.custom_flags(0o400_000 | 0o4_000);
    }
    let mut arquivo = abertura.open(caminho).map_err(|_| FalhaPdf::Leitura)?;
    let antes = arquivo.metadata().map_err(|_| FalhaPdf::Leitura)?;
    if !antes.is_file() {
        return Err(FalhaPdf::NaoRegular);
    }
    if antes.len() == 0 {
        return Err(FalhaPdf::Vazio);
    }
    if limite_observado.is_some_and(|limite| antes.len() > limite) {
        return Err(FalhaPdf::AcimaDoLimite);
    }
    let mut digest = Sha256::new();
    let mut inicio = [0_u8; 5];
    if arquivo.read_exact(&mut inicio).is_err() || inicio != *b"%PDF-" {
        return Err(FalhaPdf::Conteudo);
    }
    digest.update(inicio);
    let mut total = inicio.len() as u64;
    let mut bloco = [0_u8; 8 * 1024];
    loop {
        let lidos = arquivo.read(&mut bloco).map_err(|_| FalhaPdf::Leitura)?;
        if lidos == 0 {
            break;
        }
        total = total.checked_add(lidos as u64).ok_or(FalhaPdf::Leitura)?;
        if limite_observado.is_some_and(|limite| total > limite) {
            return Err(FalhaPdf::AcimaDoLimite);
        }
        digest.update(&bloco[..lidos]);
    }
    if total != antes.len() {
        return Err(FalhaPdf::Leitura);
    }
    Ok(PdfLocal {
        nome: nome.to_owned(),
        tamanho_bytes: total,
        sha256: crate::hex::hex_minusculo(&digest.finalize()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static PROXIMO: AtomicUsize = AtomicUsize::new(0);
    const PDF: &[u8] = b"%PDF-1.4\n1 0 obj\n%%EOF\n";
    const SHA: &str = "49286c45c90723857946342e0610a2b28974f176218f77f79efacc34cc4567bc";

    struct Pasta(PathBuf);
    impl Pasta {
        fn nova() -> Self {
            let nome = format!(
                "ies-332-pdf-{}-{}",
                std::process::id(),
                PROXIMO.fetch_add(1, Ordering::Relaxed)
            );
            let pasta = std::env::temp_dir().join(nome);
            fs::create_dir(&pasta).unwrap();
            Self(pasta)
        }
        fn arquivo(&self, nome: &str, bytes: &[u8]) -> PathBuf {
            let path = self.0.join(nome);
            fs::write(&path, bytes).unwrap();
            path
        }
    }
    impl Drop for Pasta {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn pdf_regular_publica_nome_tamanho_e_hash_sem_conteudo() {
        // Caso de documento externo observado no Todos #775 (UnB 4): PDF local.
        let pasta = Pasta::nova();
        let path = pasta.arquivo("peticao.pdf", PDF);
        assert_eq!(
            inspecionar_pdf(&path, None),
            Ok(PdfLocal {
                nome: "peticao.pdf".to_owned(),
                tamanho_bytes: 23,
                sha256: SHA.to_owned(),
            })
        );
    }

    #[test]
    fn extensao_pdf_em_maiusculas_eh_aceita() {
        let pasta = Pasta::nova();
        let path = pasta.arquivo("PETICAO.PDF", PDF);
        assert!(inspecionar_pdf(&path, None).is_ok());
    }

    #[test]
    fn arquivo_ausente_e_diretorio_sao_recusados() {
        let pasta = Pasta::nova();
        assert_eq!(
            inspecionar_pdf(&pasta.0.join("nada.pdf"), None),
            Err(FalhaPdf::Ausente)
        );
        assert_eq!(inspecionar_pdf(&pasta.0, None), Err(FalhaPdf::NaoRegular));
    }

    #[test]
    fn arquivo_vazio_ou_falso_pdf_falha_antes_da_rede() {
        let pasta = Pasta::nova();
        let vazio = pasta.arquivo("vazio.pdf", b"");
        let texto = pasta.arquivo("texto.pdf", b"conteudo arbitrario");
        assert_eq!(inspecionar_pdf(&vazio, None), Err(FalhaPdf::Vazio));
        assert_eq!(inspecionar_pdf(&texto, None), Err(FalhaPdf::Conteudo));
    }

    #[test]
    fn extensao_diferente_ou_limite_observado_recusam_o_arquivo() {
        let pasta = Pasta::nova();
        let txt = pasta.arquivo("peticao.txt", PDF);
        let pdf = pasta.arquivo("peticao.pdf", PDF);
        assert_eq!(inspecionar_pdf(&txt, None), Err(FalhaPdf::Extensao));
        assert_eq!(
            inspecionar_pdf(&pdf, Some(22)),
            Err(FalhaPdf::AcimaDoLimite)
        );
        assert!(inspecionar_pdf(&pdf, Some(23)).is_ok());
    }

    #[test]
    fn nome_com_entidade_html_rejeitado_antes_de_qualquer_upload() {
        // Oráculo: Todos documento_externo.py, NomeDeArquivoNaoAceito.
        let pasta = Pasta::nova();
        let path = pasta.arquivo("relatorio&#1.pdf", PDF);
        let erro = inspecionar_pdf(&path, None).unwrap_err();
        assert_eq!(erro.subcodigo(), "nome_arquivo_pdf_invalido");
    }

    #[cfg(unix)]
    #[test]
    fn nome_com_controle_terminal_rejeitado() {
        let pasta = Pasta::nova();
        let path = pasta.arquivo("relatorio\u{1b}[31m.pdf", PDF);
        let erro = inspecionar_pdf(&path, None).unwrap_err();
        assert_eq!(erro.subcodigo(), "nome_arquivo_pdf_invalido");
    }

    #[cfg(unix)]
    #[test]
    fn arquivo_e_ancestral_simbolicos_sao_recusados() {
        use std::os::unix::fs::symlink;
        let pasta = Pasta::nova();
        let real = pasta.arquivo("real.pdf", PDF);
        let link = pasta.0.join("link.pdf");
        symlink(&real, &link).unwrap();
        assert_eq!(inspecionar_pdf(&link, None), Err(FalhaPdf::LinkSimbolico));

        let alias = pasta.0.with_extension("alias");
        symlink(&pasta.0, &alias).unwrap();
        let via_ancestral = alias.join("real.pdf");
        assert_eq!(
            inspecionar_pdf(&via_ancestral, None),
            Err(FalhaPdf::LinkSimbolico)
        );
        fs::remove_file(alias).unwrap();
    }
}
