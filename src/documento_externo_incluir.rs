//! #332 — inspeção local do PDF, inteiramente anterior à sessão e à rede.
//! O limite da instalação só será aplicado depois de observado no cadastro.

// Núcleo de protocolo separado: ainda sem transmissão ou cadastro remoto.
pub(crate) mod anexo;
pub(crate) mod cadastro;
pub(crate) mod cadastro_envio;
pub(crate) mod confirmacao;
pub(crate) mod escolha;
pub(crate) mod fluxo;
pub(crate) mod fluxo_confirmar;
// #332: corpo multipart só será enviado após validar o formulário em treinamento.
pub(crate) mod multipart;
pub(crate) mod recibo;
pub(crate) mod upload;

/// Soma ao total os bytes lidos; estouro de `u64` é leitura impossível.
fn somar_lidos(total: u64, lidos: usize) -> Result<u64, FalhaPdf> {
    u64::try_from(lidos)
        .ok()
        .and_then(|lidos| total.checked_add(lidos))
        .ok_or(FalhaPdf::Leitura)
}

#[cfg(test)]
#[path = "documento_externo_incluir/tests_upload.rs"]
mod tests_upload;

use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read, Seek};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PdfLocal {
    pub(crate) nome: String,
    pub(crate) tamanho_bytes: u64,
    pub(crate) sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaPdf {
    Ausente,
    NaoRegular,
    LinkSimbolico,
    Vazio,
    Extensao,
    NomeInvalido,
    Conteudo,
    AcimaDoLimite,
    /// O sistema recusou consultar, abrir ou ler o arquivo (motivo do sistema).
    ArquivoIlegivel(ErrorKind),
    /// A soma dos bytes lidos não cabe em `u64`: leitura impossível.
    Leitura,
    AlteradoDuranteLeitura,
    DivergenteDaPrevia,
}

/// Não permite que a apresentação do nome/tipo seja reordenada no terminal.
/// É validação de saída, não uma restrição de charset inferida do SEI.
pub(crate) fn contem_controle_de_apresentacao(valor: &str) -> bool {
    valor.chars().any(|c| {
        c.is_control()
            || matches!(
                c,
                '\u{061c}'
                    | '\u{200e}'
                    | '\u{200f}'
                    | '\u{202a}'..='\u{202e}'
                    | '\u{2066}'..='\u{2069}'
            )
    })
}

/// Inspeção local; não prova que a instalação oferece tipo, limite ou upload.
/// Só aceita o caminho e a extensão fornecidos; não segue links simbólicos.
pub(crate) fn inspecionar_pdf(
    caminho: &Path,
    limite_observado: Option<u64>,
) -> Result<PdfLocal, FalhaPdf> {
    inspecionar_pdf_com_acao(caminho, limite_observado, || {})
}

/// Reabre o mesmo caminho após a prévia, confere o conteúdo novamente e
/// mantém aberto o exato descritor validado para o envio multipart.
pub(crate) fn abrir_pdf_para_envio(
    caminho: &Path,
    previa: &PdfLocal,
) -> Result<fs::File, FalhaPdf> {
    let (atual, arquivo) = inspecionar_pdf_com_arquivo(caminho, None, || {})?;
    if &atual != previa {
        return Err(FalhaPdf::DivergenteDaPrevia);
    }
    Ok(arquivo)
}

fn inspecionar_pdf_com_acao(
    caminho: &Path,
    limite_observado: Option<u64>,
    entre_metadata_e_leitura: impl FnOnce(),
) -> Result<PdfLocal, FalhaPdf> {
    inspecionar_pdf_com_arquivo(caminho, limite_observado, entre_metadata_e_leitura)
        .map(|(pdf, _)| pdf)
}

/// Recusa links simbólicos em qualquer nível do caminho, antes de abrir o arquivo.
fn conferir_ancestrais(caminho: &Path) -> Result<(), FalhaPdf> {
    for parte in caminho.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        let info = fs::symlink_metadata(parte).map_err(|erro| {
            if erro.kind() == ErrorKind::NotFound {
                FalhaPdf::Ausente
            } else {
                FalhaPdf::ArquivoIlegivel(erro.kind())
            }
        })?;
        if info.file_type().is_symlink() {
            return Err(FalhaPdf::LinkSimbolico);
        }
    }
    Ok(())
}

fn inspecionar_pdf_com_arquivo(
    caminho: &Path,
    limite_observado: Option<u64>,
    entre_metadata_e_leitura: impl FnOnce(),
) -> Result<(PdfLocal, fs::File), FalhaPdf> {
    // A recusa acontece antes de ler qualquer arquivo de sessão ou enviar HTTP.
    conferir_ancestrais(caminho)?;
    if !fs::symlink_metadata(caminho)
        .map_err(|erro| FalhaPdf::ArquivoIlegivel(erro.kind()))?
        .is_file()
    {
        return Err(FalhaPdf::NaoRegular);
    }
    let nome = caminho
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(FalhaPdf::Extensao)?;
    // O SEI não aceita "&#" em nome de arquivo (oráculo Todos #775).
    // Controles também não podem atravessar a apresentação humana da CLI.
    if nome.contains("&#") || contem_controle_de_apresentacao(nome) {
        return Err(FalhaPdf::NomeInvalido);
    }
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
    let mut arquivo = abertura
        .open(caminho)
        .map_err(|erro| FalhaPdf::ArquivoIlegivel(erro.kind()))?;
    let antes = arquivo
        .metadata()
        .map_err(|erro| FalhaPdf::ArquivoIlegivel(erro.kind()))?;
    if !antes.is_file() {
        return Err(FalhaPdf::NaoRegular);
    }
    if antes.len() == 0 {
        return Err(FalhaPdf::Vazio);
    }
    if limite_observado.is_some_and(|limite| antes.len() > limite) {
        return Err(FalhaPdf::AcimaDoLimite);
    }
    entre_metadata_e_leitura();
    let mut digest = Sha256::new();
    let mut inicio = [0_u8; 5];
    if arquivo.read_exact(&mut inicio).is_err() || inicio != *b"%PDF-" {
        return Err(FalhaPdf::Conteudo);
    }
    digest.update(inicio);
    let mut total = somar_lidos(0, inicio.len())?;
    let mut bloco = [0_u8; 8 * 1024];
    loop {
        let lidos = arquivo
            .read(&mut bloco)
            .map_err(|erro| FalhaPdf::ArquivoIlegivel(erro.kind()))?;
        if lidos == 0 {
            break;
        }
        total = somar_lidos(total, lidos)?;
        if limite_observado.is_some_and(|limite| total > limite) {
            return Err(FalhaPdf::AcimaDoLimite);
        }
        digest.update(bloco.split_at(lidos.min(bloco.len())).0);
    }
    // Conferir o mesmo descritor após o hash: tamanho igual não prova
    // imutabilidade, pois outro processo pode regravar bytes no meio da leitura.
    let depois = arquivo
        .metadata()
        .map_err(|erro| FalhaPdf::ArquivoIlegivel(erro.kind()))?;
    let alterado = total != antes.len()
        || depois.len() != antes.len()
        || matches!((antes.modified(), depois.modified()), (Ok(a), Ok(b)) if a != b);
    #[cfg(unix)]
    let alterado = {
        use std::os::unix::fs::MetadataExt;
        alterado || (antes.ctime(), antes.ctime_nsec()) != (depois.ctime(), depois.ctime_nsec())
    };
    if alterado {
        return Err(FalhaPdf::AlteradoDuranteLeitura);
    }
    arquivo
        .rewind()
        .map_err(|erro| FalhaPdf::ArquivoIlegivel(erro.kind()))?;
    Ok((
        PdfLocal {
            nome: nome.to_owned(),
            tamanho_bytes: total,
            sha256: hex::encode(digest.finalize()),
        },
        arquivo,
    ))
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
        assert_eq!(erro, FalhaPdf::NomeInvalido);
    }

    #[cfg(unix)]
    #[test]
    fn nome_com_controle_terminal_rejeitado() {
        let pasta = Pasta::nova();
        let path = pasta.arquivo("relatorio\u{1b}[31m.pdf", PDF);
        let erro = inspecionar_pdf(&path, None).unwrap_err();
        assert_eq!(erro, FalhaPdf::NomeInvalido);
    }

    #[test]
    fn marcadores_direcionais_unicode_no_nome_nao_podem_enganar_a_previa_humana() {
        // O cabeçalho de terminal não deve inverter caracteres ao apresentar um caminho.
        // Teste sintético: sem requisição e sem depender do charset da instalação.
        let pasta = Pasta::nova();
        for nome in [
            "relatorio\u{202e}fdp.pdf",
            "relatorio\u{2067}A.pdf",
            "relatorio\u{200f}.pdf",
        ] {
            let path = pasta.arquivo(nome, PDF);
            assert_eq!(inspecionar_pdf(&path, None), Err(FalhaPdf::NomeInvalido));
        }
    }

    #[test]
    fn pdf_do_upload_deve_ser_o_mesmo_inspecionado_na_previa() {
        // BDD-332-03, pré-condição técnica: nada de enviar bytes diferentes.
        use std::io::Read as _;
        let pasta = Pasta::nova();
        let caminho = pasta.arquivo("prova.pdf", PDF);
        let previa = inspecionar_pdf(&caminho, None).unwrap();
        let mut arquivo = abrir_pdf_para_envio(&caminho, &previa).unwrap();
        let mut bytes = Vec::new();
        arquivo.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, PDF);

        let mut adulterado = PDF.to_vec();
        adulterado[8] = b'X';
        fs::write(&caminho, &adulterado).unwrap();
        assert!(matches!(
            abrir_pdf_para_envio(&caminho, &previa),
            Err(FalhaPdf::DivergenteDaPrevia)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn pdf_substituido_por_symlink_nao_autoriza_upload() {
        use std::os::unix::fs::symlink;
        let pasta = Pasta::nova();
        let caminho = pasta.arquivo("prova.pdf", PDF);
        let previa = inspecionar_pdf(&caminho, None).unwrap();
        let outro = pasta.arquivo("outro.pdf", PDF);
        fs::remove_file(&caminho).unwrap();
        symlink(&outro, &caminho).unwrap();
        assert!(matches!(
            abrir_pdf_para_envio(&caminho, &previa),
            Err(FalhaPdf::LinkSimbolico)
        ));
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

    #[test]
    fn escrita_concorrente_de_mesmo_tamanho_nao_pode_gerar_hash_de_foto_inconsistente() {
        // A data foi fixada no passado: a mudança é detectável sem sleeps/flakiness.
        let pasta = Pasta::nova();
        let caminho = pasta.arquivo("concorrente.pdf", PDF);
        let data_antiga = std::fs::FileTimes::new().set_modified(std::time::UNIX_EPOCH);
        std::fs::OpenOptions::new()
            .write(true)
            .open(&caminho)
            .unwrap()
            .set_times(data_antiga)
            .unwrap();
        let resultado = inspecionar_pdf_com_acao(&caminho, None, || {
            fs::write(&caminho, b"%PDF-1.4\n2 0 obj\n%%EOF\n").unwrap();
        });
        assert_eq!(resultado, Err(FalhaPdf::AlteradoDuranteLeitura));
    }
}
