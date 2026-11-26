//! Texto de PDF local, OCR nas páginas sem texto nativo; cobertura parcial.
use super::{ErroAnexo, conferir_token};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ExtracaoPdf {
    NativoParcial(String),
    NativoComDiagnostico(String),
    MistoParcial(String),
    OcrParcial(String),
    MistoComDiagnostico(String),
    OcrComDiagnostico(String),
    SemTextoNativo,
    OcrIndisponivel,
    MotorAusente,
    Falhou,
}

impl ExtracaoPdf {
    pub(crate) fn texto(&self) -> Option<&str> {
        match self {
            Self::NativoParcial(texto)
            | Self::NativoComDiagnostico(texto)
            | Self::MistoParcial(texto)
            | Self::OcrParcial(texto)
            | Self::MistoComDiagnostico(texto)
            | Self::OcrComDiagnostico(texto) => Some(texto),
            Self::SemTextoNativo | Self::OcrIndisponivel | Self::MotorAusente | Self::Falhou => None,
        }
    }

    pub(crate) fn estado(&self) -> &'static str {
        match self {
            Self::NativoParcial(_) => "nativo_parcial",
            Self::NativoComDiagnostico(_) => "nativo_com_diagnostico",
            Self::MistoParcial(_) => "misto_parcial",
            Self::OcrParcial(_) => "ocr_parcial",
            Self::MistoComDiagnostico(_) => "misto_com_diagnostico",
            Self::OcrComDiagnostico(_) => "ocr_com_diagnostico",
            Self::SemTextoNativo => "sem_texto_nativo",
            Self::OcrIndisponivel => "ocr_indisponivel",
            Self::MotorAusente => "motor_ausente",
            Self::Falhou => "falhou",
        }
    }
}

pub(super) fn extrair(caminho: &Path) -> Result<ExtracaoPdf, ErroAnexo> {
    let prazo = Instant::now() + Duration::from_secs(30);
    let limite = crate::MAX_HTML_BYTES;
    let mut comando = Command::new("pdftotext");
    comando.args(["-enc", "UTF-8"]).arg(caminho).arg("-");
    let (bytes, diagnostico) = match executar(&mut comando, &[], limite, prazo) {
        Ok(saida) => saida,
        Err(ExtracaoPdf::MotorAusente) => return Ok(ExtracaoPdf::MotorAusente),
        Err(_) => return Ok(ExtracaoPdf::Falhou),
    };
    let nativo = classificar(&bytes, diagnostico)?;
    if matches!(
        nativo,
        ExtracaoPdf::Falhou | ExtracaoPdf::NativoComDiagnostico(_)
    ) {
        return Ok(nativo);
    }
    let Ok(texto) = std::str::from_utf8(&bytes) else {
        return Ok(nativo);
    };
    complementar(caminho, texto, nativo, prazo)
}

fn complementar(
    caminho: &Path,
    texto: &str,
    nativo: ExtracaoPdf,
    prazo: Instant,
) -> Result<ExtracaoPdf, ErroAnexo> {
    let mut recuperado = String::new();
    let mut houve_ocr = false;
    let mut incompleto = false;
    for (indice, pagina) in texto.split_terminator('\u{c}').enumerate() {
        let parte = if pagina.trim().is_empty() {
            if Instant::now() >= prazo {
                incompleto = true;
                continue;
            }
            match ocr(caminho, indice + 1, prazo) {
                Ok(bytes) => match classificar(&bytes, false)? {
                    ExtracaoPdf::NativoParcial(texto) => {
                        houve_ocr = true;
                        texto
                    }
                    ExtracaoPdf::SemTextoNativo => continue,
                    _ => {
                        incompleto = true;
                        continue;
                    }
                },
                Err(_) => {
                    incompleto = true;
                    continue;
                }
            }
        } else {
            pagina.trim().to_owned()
        };
        if recuperado.len().saturating_add(parte.len()).saturating_add(1) > crate::MAX_HTML_BYTES {
            return Ok(ExtracaoPdf::Falhou);
        }
        recuperado.push_str(&parte);
        recuperado.push('\n');
    }
    if !houve_ocr {
        return Ok(match (nativo, incompleto) {
            (ExtracaoPdf::SemTextoNativo, true) => ExtracaoPdf::OcrIndisponivel,
            (ExtracaoPdf::NativoParcial(texto), true) => ExtracaoPdf::NativoComDiagnostico(texto),
            (estado, _) => estado,
        });
    }
    let ExtracaoPdf::NativoParcial(recuperado) = classificar(recuperado.as_bytes(), false)? else {
        return Ok(ExtracaoPdf::Falhou);
    };
    Ok(match (nativo.texto().is_some(), incompleto) {
        (true, false) => ExtracaoPdf::MistoParcial(recuperado),
        (false, false) => ExtracaoPdf::OcrParcial(recuperado),
        (true, true) => ExtracaoPdf::MistoComDiagnostico(recuperado),
        (false, true) => ExtracaoPdf::OcrComDiagnostico(recuperado),
    })
}

fn ocr(caminho: &Path, pagina: usize, prazo: Instant) -> Result<Vec<u8>, ExtracaoPdf> {
    let numero = pagina.to_string();
    let mut raster = Command::new("pdftoppm");
    raster
        .args([
            "-f",
            &numero,
            "-l",
            &numero,
            "-singlefile",
            "-gray",
            "-scale-to",
            "2000",
        ])
        .arg(caminho);
    let (imagem, diagnostico) = executar(&mut raster, &[], crate::MAX_HTML_BYTES, prazo)?;
    if diagnostico || imagem.is_empty() {
        return Err(ExtracaoPdf::Falhou);
    }
    let mut motor = Command::new("tesseract");
    motor.args(["stdin", "stdout", "-l", "por"]);
    let (texto, _) = executar(&mut motor, &imagem, crate::MAX_HTML_BYTES, prazo)?;
    Ok(texto)
}

fn executar(
    comando: &mut Command,
    entrada: &[u8],
    limite: usize,
    prazo: Instant,
) -> Result<(Vec<u8>, bool), ExtracaoPdf> {
    if Instant::now() >= prazo {
        return Err(ExtracaoPdf::Falhou);
    }
    let mut filho = match comando
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(filho) => filho,
        Err(erro) if erro.kind() == io::ErrorKind::NotFound => {
            return Err(ExtracaoPdf::MotorAusente);
        }
        Err(_) => return Err(ExtracaoPdf::Falhou),
    };
    let (Some(mut stdin), Some(stdout), Some(mut stderr)) =
        (filho.stdin.take(), filho.stdout.take(), filho.stderr.take())
    else {
        let _ = filho.kill();
        let _ = filho.wait();
        return Err(ExtracaoPdf::Falhou);
    };
    let resultado = std::thread::scope(|escopo| {
        let escrita = escopo.spawn(move || stdin.write_all(entrada));
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
        escrita.join().ok()?.ok()?;
        let bytes = texto.join().ok()?.ok()?;
        let diagnostico = diagnostico.join().ok()?.ok()? > 0;
        Some((status?, bytes, diagnostico))
    });
    let Some((status, bytes, diagnostico)) = resultado else {
        return Err(ExtracaoPdf::Falhou);
    };
    if !status.success() || bytes.len() > limite {
        return Err(ExtracaoPdf::Falhou);
    }
    Ok((bytes, diagnostico))
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
        assert_eq!(
            classificar(b"\n\x0c", false),
            Ok(ExtracaoPdf::SemTextoNativo)
        );
        assert_eq!(classificar(b"", true), Ok(ExtracaoPdf::Falhou));
        assert_eq!(classificar(b"\xff", false), Ok(ExtracaoPdf::Falhou));
        assert_eq!(
            classificar(b"infra_hash=FICTICIO", false),
            Err(ErroAnexo::ConteudoComLinkAssinado)
        );
    }
}
