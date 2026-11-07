//! #332 — inspeção local do PDF, inteiramente anterior à sessão e à rede.
//! O limite da instalação só será aplicado depois de observado no cadastro.

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
pub(crate) fn inspecionar_pdf(
    _caminho: &Path,
    _limite_observado: Option<u64>,
) -> Result<PdfLocal, FalhaPdf> {
    // Implementação só após RED dos testes sintéticos.
    Err(FalhaPdf::Ausente)
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
        assert_eq!(inspecionar_pdf(&pdf, Some(22)), Err(FalhaPdf::AcimaDoLimite));
        assert!(inspecionar_pdf(&pdf, Some(23)).is_ok());
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