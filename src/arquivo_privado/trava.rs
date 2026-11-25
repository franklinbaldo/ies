//! Trava exclusiva de arquivo com espera ativa e prazo (`flock` da `std`).
//! Usada pela preparação do índice local e pelo login: a trava vive
//! enquanto o [`TravaDeArquivo`] viver.
use std::fs::{File, TryLockError};
use std::time::{Duration, Instant};

/// Trava exclusiva de um arquivo; liberada quando sai de escopo.
#[derive(Debug)]
pub(crate) struct TravaDeArquivo {
    _arquivo: File,
}

/// Por que a trava não foi obtida. Não carrega caminho nem conteúdo.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDaTrava {
    /// Outro handle segura a trava até o prazo.
    Ocupada,
    /// O sistema recusou o `try_lock` por outra causa que não a ocupação.
    Ilegivel(#[source] std::io::Error),
}

/// Tenta o `try_lock` de `arquivo` a cada `passo` até `espera`; depois
/// disso, sem obter, devolve [`ErroDaTrava::Ocupada`].
pub(crate) fn travar_com_espera(
    arquivo: File,
    espera: Duration,
    passo: Duration,
) -> Result<TravaDeArquivo, ErroDaTrava> {
    let limite = Instant::now() + espera;
    loop {
        match arquivo.try_lock() {
            Ok(()) => return Ok(TravaDeArquivo { _arquivo: arquivo }),
            Err(TryLockError::WouldBlock) if Instant::now() < limite => {
                std::thread::sleep(passo);
            }
            Err(TryLockError::WouldBlock) => return Err(ErroDaTrava::Ocupada),
            Err(TryLockError::Error(erro)) => return Err(ErroDaTrava::Ilegivel(erro)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::path::PathBuf;

    const PRAZO_CURTO: Duration = Duration::from_millis(30);
    const PASSO: Duration = Duration::from_millis(5);

    /// Arquivo temporário do teste, removido ao sair de escopo.
    struct Arquivo(PathBuf);

    impl Arquivo {
        fn novo(nome: &str) -> Self {
            let caminho =
                std::env::temp_dir().join(format!("ies-trava-{nome}-{}", std::process::id()));
            Self(caminho)
        }

        fn abrir(&self) -> File {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&self.0)
                .unwrap()
        }
    }

    impl Drop for Arquivo {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn arquivo_livre_devolve_a_trava() {
        let arquivo = Arquivo::novo("livre");
        let trava = travar_com_espera(arquivo.abrir(), PRAZO_CURTO, PASSO);
        assert!(trava.is_ok());
    }

    #[test]
    fn arquivo_travado_por_outro_handle_devolve_ocupada_apos_o_prazo() {
        let arquivo = Arquivo::novo("ocupada");
        let ocupante = arquivo.abrir();
        ocupante.try_lock().unwrap();
        let inicio = Instant::now();
        let resultado = travar_com_espera(arquivo.abrir(), PRAZO_CURTO, PASSO);
        assert!(matches!(resultado, Err(ErroDaTrava::Ocupada)));
        assert!(inicio.elapsed() >= PRAZO_CURTO);
    }
}
