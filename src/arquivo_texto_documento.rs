//! Persistência privada de texto de documento já verificado offline.
//! Nunca sobrescreve destino existente, não publica conteúdo em diagnóstico.
use crate::arquivo_privado::FalhaDaGravacao;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroArquivoTexto {
    NomeAusente,
    DiretorioAusente,
    DestinoJaExiste,
    EscritaFalhou,
}
/// A falha da gravação privada única nos erros do texto.
fn traduzir(falha: crate::arquivo_privado::FalhaDaGravacao) -> ErroArquivoTexto {
    match falha {
        FalhaDaGravacao::SemNomeDeArquivo => ErroArquivoTexto::NomeAusente,
        FalhaDaGravacao::DiretorioAusente
        | FalhaDaGravacao::DiretorioNaoConsultado(_)
        | FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio
        | FalhaDaGravacao::NivelDoCaminhoIlegivel(_) => ErroArquivoTexto::DiretorioAusente,
        FalhaDaGravacao::DestinoJaExiste => ErroArquivoTexto::DestinoJaExiste,
        // Diretório do usuário: a peça não confere o modo.
        #[cfg(unix)]
        FalhaDaGravacao::DiretorioGravavelPorOutros => ErroArquivoTexto::EscritaFalhou,
        FalhaDaGravacao::DestinoNaoEhArquivoRegular
        | FalhaDaGravacao::AcimaDoLimite
        | FalhaDaGravacao::CriacaoFalhou(_)
        | FalhaDaGravacao::CriacaoDoDiretorioFalhou(_)
        | FalhaDaGravacao::CriacaoDoTemporarioFalhou(_)
        | FalhaDaGravacao::NomesDeTemporarioEsgotados
        | FalhaDaGravacao::EscritaFalhou
        | FalhaDaGravacao::ErroDeIoNaEscrita(_)
        | FalhaDaGravacao::ConfirmacaoFalhou(_) => ErroArquivoTexto::EscritaFalhou,
    }
}

/// O diretório do destino existe e é diretório (links no caminho são
/// aceitos; o modo é da pessoa): a conferência da gravação privada única
/// para o diretório do usuário. Não estabelece exclusividade contra
/// alterações concorrentes do caminho.
pub(crate) fn conferir_diretorios(destino: &Path) -> Result<(), ErroArquivoTexto> {
    use crate::arquivo_privado::Politica;
    crate::arquivo_privado::conferir_diretorios(
        destino,
        Politica::exclusiva().diretorio_do_usuario(),
    )
    .map_err(traduzir)
}

pub fn salvar(destino: &Path, texto: &str) -> Result<usize, ErroArquivoTexto> {
    use crate::arquivo_privado::{self, Politica};
    if destino.file_name().is_none() {
        return Err(ErroArquivoTexto::NomeAusente);
    }
    // Como antes da #211: nome sem diretório (pai vazio) é recusado.
    if !destino.parent().is_some_and(Path::is_dir) {
        return Err(ErroArquivoTexto::DiretorioAusente);
    }
    arquivo_privado::gravar(
        destino,
        texto.as_bytes(),
        Politica::exclusiva().diretorio_do_usuario(),
    )
    .map_err(traduzir)?;
    Ok(texto.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn pasta() -> std::path::PathBuf {
        let pasta = std::env::temp_dir().join(format!(
            "ies-texto-documento-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&pasta).unwrap();
        pasta
    }

    #[test]
    fn salva_texto_exato_em_arquivo_privado_novo_sem_sobrescrever() {
        let pasta = pasta();
        let destino = pasta.join("0007.txt");
        assert_eq!(
            salvar(&destino, "Decisão\nAssinada"),
            Ok("Decisão\nAssinada".len())
        );
        assert_eq!(fs::read_to_string(&destino).unwrap(), "Decisão\nAssinada");
        assert_eq!(
            salvar(&destino, "OUTRO"),
            Err(ErroArquivoTexto::DestinoJaExiste)
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&destino).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn ausencia_de_diretorio_tem_erro_tipado_e_nao_cria_destino() {
        let pasta = pasta();
        let destino = pasta.join("inexistente/0007.txt");
        assert_eq!(
            salvar(&destino, "conteúdo"),
            Err(ErroArquivoTexto::DiretorioAusente)
        );
        assert!(!destino.exists());
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn aceita_link_simbolico_como_diretorio_de_saida_e_em_ancestral() {
        use std::os::unix::fs::symlink;
        let raiz = pasta();
        let real = raiz.join("real");
        let subpasta = real.join("subpasta");
        fs::create_dir_all(&subpasta).unwrap();
        let link = raiz.join("atalho");
        symlink(&real, &link).unwrap();
        assert_eq!(salvar(&link.join("0007.txt"), "DIR"), Ok(3));
        assert_eq!(
            salvar(&link.join("subpasta").join("0007.txt"), "ANC"),
            Ok(3)
        );
        assert_eq!(fs::read_to_string(real.join("0007.txt")).unwrap(), "DIR");
        assert_eq!(
            fs::read_to_string(subpasta.join("0007.txt")).unwrap(),
            "ANC"
        );
        fs::remove_dir_all(raiz).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn aceita_diretorio_de_saida_gravavel_pelo_grupo_ou_por_outros() {
        // `--saida DIR` é da pessoa: o modo do diretório é dela.
        use std::os::unix::fs::PermissionsExt;
        let raiz = pasta();
        let aberto = raiz.join("aberto");
        fs::create_dir(&aberto).unwrap();
        for bits in [0o775, 0o777] {
            fs::set_permissions(&aberto, fs::Permissions::from_mode(bits)).unwrap();
            let destino = aberto.join(format!("{bits:o}.txt"));
            assert_eq!(conferir_diretorios(&destino), Ok(()));
            assert_eq!(salvar(&destino, "OK"), Ok(2));
            assert_eq!(
                salvar(&destino, "DE NOVO"),
                Err(ErroArquivoTexto::DestinoJaExiste)
            );
        }
        fs::remove_dir_all(raiz).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn destino_simbolico_nao_pode_alterar_arquivo_existente() {
        use std::os::unix::fs::symlink;
        let pasta = pasta();
        let original = pasta.join("original.txt");
        fs::write(&original, "NAO SUBSTITUIR").unwrap();
        let destino = pasta.join("texto.txt");
        symlink(&original, &destino).unwrap();
        assert_eq!(
            salvar(&destino, "INTRUSO"),
            Err(ErroArquivoTexto::DestinoJaExiste)
        );
        assert_eq!(fs::read_to_string(&original).unwrap(), "NAO SUBSTITUIR");
        fs::remove_dir_all(pasta).unwrap();
    }
}
