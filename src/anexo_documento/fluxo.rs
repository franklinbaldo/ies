//! Fluxo de leitura do corpo do anexo: erro de transporte e texto.
//! O estado de arquivo continua nos primitivos privados já compartilhados.
use super::{
    AnexoLido, BLOCO, ConteudoDoAnexo, DestinoDoAnexo, ErroAnexo, LimiteDoAnexo, Read,
    RespostaBinaria, abrir_parcial, abrir_parcial_cache, cabecalhos, conferir, conferir_token,
    erro_ao_escrever, erro_de_leitura, publicar_cache,
};
use crate::transporte::{RespostaWeb, recusar_status};
use sha2::{Digest, Sha256};

/// Lê a resposta de `documento_download_anexo` do documento `numero`.
/// 3xx não chega aqui: quem chama o trata como no resto do fluxo.
pub(crate) fn ler_anexo<R: Read>(
    resposta: RespostaBinaria<R>,
    numero: &str,
    limite: LimiteDoAnexo,
    destino: DestinoDoAnexo<'_>,
) -> Result<AnexoLido, ErroAnexo> {
    if resposta.status != 200 {
        // As mesmas recusas de status do transporte (corpo até 64 KiB).
        let mime = resposta
            .valores("content-type")
            .first()
            .map_or_else(String::new, |v| (*v).to_owned());
        return Err(ErroAnexo::Transporte(recusar_status(RespostaWeb {
            status_code: i32::from(resposta.status),
            mime,
            content_length: None,
            localizacao: None,
            corpo: resposta.corpo,
        })));
    }
    let mut conferido = conferir(&resposta, limite)?;
    let RespostaBinaria { corpo, .. } = resposta;
    let mut arquivo = match destino {
        DestinoDoAnexo::SoMetadados => None,
        DestinoDoAnexo::Diretorio(dir) => Some(abrir_parcial(dir, numero, &conferido)?),
        DestinoDoAnexo::Cache { diretorio, .. } => {
            Some(abrir_parcial_cache(diretorio, numero, &conferido)?)
        }
    };
    let texto = cabecalhos::e_texto(&conferido.mime);
    let mut guardado = Vec::new();
    let mut hasher = Sha256::new();
    let mut tamanho = 0u64;
    // O token não pode chegar ao cache, mesmo se cruzar dois blocos.
    let mut token_lido = 0usize;
    let mut leitor = corpo.take(conferido.limite.saturating_add(1));
    let mut bloco = vec![0u8; BLOCO];
    loop {
        let lido = match leitor.read(&mut bloco) {
            Ok(0) => break,
            Ok(lido) => lido,
            Err(erro) if erro.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(erro) => {
                return Err(erro_de_leitura(&erro));
            }
        };
        tamanho = u64::try_from(lido)
            .ok()
            .and_then(|lido| tamanho.checked_add(lido))
            .ok_or(ErroAnexo::AcimaDoLimite)?;
        if tamanho > conferido.limite {
            return Err(ErroAnexo::AcimaDoLimite);
        }
        let (lidos, _) = bloco.split_at(lido.min(bloco.len()));
        conferir_token(lidos, &mut token_lido)?;
        hasher.update(lidos);
        if let Some(aberto) = &mut arquivo {
            aberto.parcial.escrever(lidos).map_err(erro_ao_escrever)?;
        }
        if texto {
            guardado.extend_from_slice(lidos);
        }
    }
    if tamanho == 0 {
        return Err(ErroAnexo::Vazio);
    }
    if conferido.mime == "text/html" {
        conferir_html(&guardado, &conferido.valor_do_tipo)?;
    }
    // Depois do corpo, como o login e a exceção do SEI.
    let (disposicao, nome_original) = conferido
        .disposicao
        .take()
        .ok_or(ErroAnexo::CabecalhoDeDisposicaoAusente)?;
    if let Some(aberto) = &mut arquivo {
        aberto
            .parcial
            .sincronizar()
            .map_err(ErroAnexo::GravacaoDoParcialFalhou)?;
    }
    let hash = hex::encode(hasher.finalize());
    publicar_cache(&mut arquivo, destino, numero, &hash, conferido.extensao)?;
    let conteudo = match arquivo {
        Some(parcial) => ConteudoDoAnexo::Arquivo(parcial),
        None if texto => ConteudoDoAnexo::Texto(texto_do_anexo(
            &conferido.mime,
            &guardado,
            &conferido.valor_do_tipo,
        )?),
        None => ConteudoDoAnexo::Nenhum,
    };
    Ok(AnexoLido {
        mime: conferido.mime,
        tamanho,
        sha256: hash,
        nome_original,
        disposicao,
        conteudo,
    })
}

fn conferir_html(bytes: &[u8], tipo: &str) -> Result<(), ErroAnexo> {
    use crate::transporte::corpo_de_recusa::decodificar;
    use crate::transporte::texto_de_recusa::{e_excecao_do_sei, texto_do_html};
    let html = decodificar(bytes, tipo);
    if crate::tela_login::e_tela_login(&html) {
        return Err(ErroAnexo::TelaDeLogin(crate::erro_publico::SessaoEncerrada));
    }
    if e_excecao_do_sei(&html) {
        let mensagem = texto_do_html(&html).and_then(|t| crate::transporte::saneamento::sanear(&t));
        return Err(ErroAnexo::RecusadoPeloSei(mensagem));
    }
    Ok(())
}

/// O texto publicado: `text/plain` no charset declarado (ISO-8859-1) ou
/// UTF-8 estrito, `\r\n` → `\n` e outros controles (fora `\n`, `\t`)
/// → `�`; `text/html` pelo texto visível do corpo.
fn texto_do_anexo(mime: &str, bytes: &[u8], tipo: &str) -> Result<String, ErroAnexo> {
    let declarado = tipo.to_ascii_lowercase();
    let latin1 = ["8859-1", "latin1", "latin-1", "windows-1252", "cp1252"]
        .iter()
        .any(|nome| declarado.contains(nome));
    let texto = if latin1 {
        bytes.iter().copied().map(char::from).collect()
    } else {
        std::str::from_utf8(bytes)
            .map_err(ErroAnexo::CorpoNaoUtf8NoCharsetDeclarado)?
            .to_owned()
    };
    if mime == "text/html" {
        return crate::documento_web::texto_do_corpo(&texto).map_err(ErroAnexo::Corpo);
    }
    Ok(texto
        .replace("\r\n", "\n")
        .chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\t' {
                '\u{fffd}'
            } else {
                c
            }
        })
        .collect())
}
