//! A foto `DOC.vN.html` de `ies documento DOC --saida DIR` (#207;
//! contracts/cli/documento-obter-versoes.md): o HTML que o SEI serviu da
//! versão N, um arquivo por versão, sem `.txt` nem arquivos ocultos. Uma
//! versão fechada não muda, então a gravação é idempotente: ausente,
//! grava (0600, sem seguir link, atômica); igual pelo SHA-256, não grava;
//! diferente, recusa tipada.
use crate::arquivo_copia::ler_regular;
use crate::arquivo_texto_documento::conferir_diretorios;
use crate::copia_documento::{ErroCopia, pasta_segura};
use crate::documento::NumeroDocumento;
use crate::documento_versoes::VersaoDocumento;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// `DOC.vN.html`.
pub(crate) fn nome(numero: &str, versao: VersaoDocumento) -> String {
    format!("{numero}.v{}.html", versao.numero())
}

/// SHA-256 em hexadecimal minúsculo.
pub(crate) fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// O resultado da gravação de uma foto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FotoGravada {
    /// `DOC.vN.html`, relativo a `--saida`.
    pub(crate) nome: String,
    pub(crate) bytes: usize,
    pub(crate) sha256: String,
    /// `false`: o arquivo já existia com o mesmo SHA-256 (no-op).
    pub(crate) gravado: bool,
}

/// `DIR` existe e é diretório (links no caminho, inclusive o próprio
/// `DIR`, são aceitos); o modo é da pessoa e não é conferido.
pub(crate) fn conferir_pasta(pasta: &Path) -> Result<(), ErroCopia> {
    conferir_diretorios(&pasta.join("x"))?;
    let real = fs::canonicalize(pasta).map_err(|erro| {
        if erro.kind() == ErrorKind::NotFound {
            ErroCopia::DestinoInseguro
        } else {
            ErroCopia::ConsultaDoDestinoFalhou(erro.kind())
        }
    })?;
    pasta_segura(&real)
}

/// Os bytes da foto já presente, lidos sem seguir link e só se privada;
/// `None` quando não há nada com esse nome.
pub(crate) fn existente(pasta: &Path, nome: &str) -> Result<Option<Vec<u8>>, ErroCopia> {
    let alvo = pasta.join(nome);
    match fs::symlink_metadata(&alvo) {
        Err(erro) if erro.kind() == ErrorKind::NotFound => Ok(None),
        Err(erro) => Err(ErroCopia::ConsultaDaFotoFalhou(erro.kind())),
        Ok(_) => ler_regular(&alvo, ErroCopia::LeituraFalhou).map(Some),
    }
}

/// A foto local declara o mesmo documento e, se servir rodapé, a mesma
/// versão do nome.
fn coerente(bytes: &[u8], numero: &str, versao: VersaoDocumento) -> bool {
    let Ok(html) = std::str::from_utf8(bytes) else {
        return false;
    };
    let mesmo_documento =
        crate::documento_web::identidade_do_titulo(html).is_ok_and(|id| id.numero == numero);
    let mesma_versao =
        crate::rodape_do_documento::versao_do_rodape(html).is_none_or(|servida| servida == versao);
    mesmo_documento && mesma_versao
}

/// Antes de qualquer rede: `DIR` seguro e, com `--versao N` (o nome já
/// é conhecido), cada `DOC.vN.html` presente legível, privado e coerente
/// com o nome (título do mesmo número SEI, rodapé da mesma versão).
pub(crate) fn conferir_antes_da_rede(
    pasta: &Path,
    numeros: &[NumeroDocumento],
    pedida: Option<VersaoDocumento>,
) -> Result<(), ErroCopia> {
    conferir_pasta(pasta)?;
    let Some(versao) = pedida else {
        return Ok(());
    };
    for numero in numeros {
        if let Some(bytes) = existente(pasta, &nome(numero.as_str(), versao))?
            && !coerente(&bytes, numero.as_str(), versao)
        {
            return Err(ErroCopia::FotoDivergente);
        }
    }
    Ok(())
}

/// Sem gravar: o que [`gravar`] faria (`gravado: false` se já igual) ou
/// a recusa, para conferir o lote inteiro antes da primeira gravação.
pub(crate) fn planejar(
    pasta: &Path,
    numero: &str,
    versao: VersaoDocumento,
    html: &str,
) -> Result<FotoGravada, ErroCopia> {
    let nome = nome(numero, versao);
    let sha256 = sha256(html.as_bytes());
    let gravado = match existente(pasta, &nome)? {
        None => true,
        Some(bytes) if self::sha256(&bytes) == sha256 => false,
        Some(_) => return Err(ErroCopia::FotoDivergente),
    };
    Ok(FotoGravada {
        nome,
        bytes: html.len(),
        sha256,
        gravado,
    })
}

/// Grava a foto pela gravação privada única, no modo exclusivo:
/// parcial `0600` no mesmo `DIR`, `fsync` e `hard_link` para o nome final
/// (falha se o nome surgiu: nunca sobrescreve); o parcial é removido.
pub(crate) fn gravar(
    pasta: &Path,
    numero: &str,
    versao: VersaoDocumento,
    html: &str,
) -> Result<FotoGravada, ErroCopia> {
    use crate::arquivo_privado::{self, FalhaDaGravacao, Politica};
    conferir_pasta(pasta)?;
    let plano = planejar(pasta, numero, versao, html)?;
    if !plano.gravado {
        return Ok(plano);
    }
    let destino = pasta.join(&plano.nome);
    match arquivo_privado::gravar(
        &destino,
        html.as_bytes(),
        Politica::exclusiva().diretorio_do_usuario(),
    ) {
        Ok(()) => Ok(plano),
        // Surgiu entre a conferência e a ligação: igual é no-op.
        Err(FalhaDaGravacao::DestinoJaExiste) => {
            let de_novo = planejar(pasta, numero, versao, html)?;
            if de_novo.gravado {
                return Err(ErroCopia::GravacaoFalhou);
            }
            Ok(de_novo)
        }
        Err(falha) => Err(ErroCopia::GravacaoDaFotoRecusada(falha)),
    }
}

#[cfg(test)]
#[path = "foto_documento/tests.rs"]
pub(crate) mod tests;
