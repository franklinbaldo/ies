//! Primeira fatia local: texto nativo de PDF, sem afirmar cobertura de OCR.
use super::{ErroAnexo, conferir_token};
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ExtracaoPdf {
    NativoParcial(String),
    NativoComDiagnostico(String),
    SemTextoNativo,
    MotorAusente,
    Falhou,
}

impl ExtracaoPdf {
    pub(crate) fn texto(&self) -> Option<&str> {
        match self {
            Self::NativoParcial(texto) | Self::NativoComDiagnostico(texto) => Some(texto),
            Self::SemTextoNativo | Self::MotorAusente | Self::Falhou => None,
        }
    }

    pub(crate) fn estado(&self) -> &'static str {
        match self {
            Self::NativoParcial(_) => "nativo_parcial",
            Self::NativoComDiagnostico(_) => "nativo_com_diagnostico",
            Self::SemTextoNativo => "sem_texto_nativo",
            Self::MotorAusente => "motor_ausente",
            Self::Falhou => "falhou",
        }
    }
}

pub(super) fn extrair(caminho: &Path) -> Result<ExtracaoPdf, ErroAnexo> {
    let mut filho = match Command::new("pdftotext")
        .args(["-enc", "UTF-8"])
        .arg(caminho)
        .arg("-")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(filho) => filho,
        Err(erro) if erro.kind() == io::ErrorKind::NotFound => {
            return Ok(ExtracaoPdf::MotorAusente);
        }
        Err(_) => return Ok(ExtracaoPdf::Falhou),
    };
    let (Some(stdout), Some(mut stderr)) = (filho.stdout.take(), filho.stderr.take()) else {
        let _ = filho.kill();
        let _ = filho.wait();
        return Ok(ExtracaoPdf::Falhou);
    };
    let limite = crate::MAX_HTML_BYTES;
    let prazo = Instant::now() + Duration::from_secs(30);
    let resultado = std::thread::scope(|escopo| {
        let texto = escopo.spawn(move || {
            let mut bytes = Vec::new();
            stdout
                .take(u64::try_from(limite).unwrap_or(u64::MAX).saturating_add(1))
                .read_to_end(&mut bytes)?;
            Ok::<_, io::Error>(bytes)
        });
        let diagnostico = escopo.spawn(move || io::copy(&mut stderr, &mut io::sink()));
        let status = loop {
            match filho.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if Instant::now() < prazo => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(None) | Err(_) => {
                    let _ = filho.kill();
                    let _ = filho.wait();
                    break None;
                }
            }
        };
        let bytes = texto.join().ok()?.ok()?;
        let diagnostico = diagnostico.join().ok()?.ok()? > 0;
        Some((status?, bytes, diagnostico))
    });
    let Some((status, bytes, diagnostico)) = resultado else {
        return Ok(ExtracaoPdf::Falhou);
    };
    if !status.success() || bytes.len() > limite {
        return Ok(ExtracaoPdf::Falhou);
    }
    classificar(&bytes, diagnostico)
}

fn classificar(bytes: &[u8], diagnostico: bool) -> Result<ExtracaoPdf, ErroAnexo> {
    let mut token_lido = 0;
    conferir_token(bytes, &mut token_lido)?;
    let Ok(texto) = std::str::from_utf8(bytes) else {
        return Ok(ExtracaoPdf::Falhou);
    };
    let texto: String = texto
        .chars()
        .map(|c| match c {
            '\u{c}' => '\n',
            c if c.is_control() && c != '\n' && c != '\t' => '\u{fffd}',
            c => c,
        })
        .collect();
    let texto = texto.trim().to_owned();
    Ok(match (texto.is_empty(), diagnostico) {
        (false, false) => ExtracaoPdf::NativoParcial(texto),
        (false, true) => ExtracaoPdf::NativoComDiagnostico(texto),
        (true, false) => ExtracaoPdf::SemTextoNativo,
        (true, true) => ExtracaoPdf::Falhou,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cobertura_e_diagnostico_nao_sao_teor_integral() {
        assert_eq!(
            classificar(b"NATIVO ALFA\n\x0c", false),
            Ok(ExtracaoPdf::NativoParcial("NATIVO ALFA".to_owned()))
        );
        assert_eq!(
            classificar(b"NATIVO ALFA", true),
            Ok(ExtracaoPdf::NativoComDiagnostico("NATIVO ALFA".to_owned()))
        );
        assert_eq!(classificar(b"\n\x0c", false), Ok(ExtracaoPdf::SemTextoNativo));
        assert_eq!(classificar(b"", true), Ok(ExtracaoPdf::Falhou));
        assert_eq!(classificar(b"\xff", false), Ok(ExtracaoPdf::Falhou));
        assert_eq!(
            classificar(b"infra_hash=FICTICIO", false),
            Err(ErroAnexo::ConteudoComLinkAssinado)
        );
    }
}
