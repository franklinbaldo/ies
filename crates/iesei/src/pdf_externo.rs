//! Estado tipado do PDF externo simulado, comum ao IES e ao anti-IES.
//! O upload temporário e o cadastro definitivo são transições independentes.
use crate::documentos_simulados as documentos;
use crate::instalacao::Instalacao;
pub use crate::documentos_simulados::Processo;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TipoExterno {
    pub id: String,
    pub nome: String,
    pub limite_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Temporario {
    pub id: i64,
    pub nome: String,
    pub tamanho: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Documento {
    pub id: i64,
    pub numero_sei: String,
    pub id_serie: String,
    pub tipo: String,
    pub nome: String,
    pub tamanho: u64,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    EntradaInvalida,
    ProcessoAusente,
    TipoAusente,
    UploadAusente,
    LimiteExcedido,
}

impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())
}

fn texto_valido(texto: &str) -> bool {
    !texto.trim().is_empty() && !texto.contains(['\r', '\n', '±', '#'])
}

fn numero(tamanho: u64) -> Result<i64, Falha> {
    i64::try_from(tamanho).map_err(|_| Falha::EntradaInvalida)
}

/// Semeia uma hipótese sintética, sempre isolada por origem e cenário.
///
/// # Errors
/// Preserva a causa SQLite e recusa identidades ou limites inválidos.
pub fn semear(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    processo: &Processo,
    tipo: &TipoExterno,
) -> Result<(), Falha> {
    if !texto_valido(cenario)
        || !id_valido(&processo.id)
        || !texto_valido(&processo.nup)
        || !texto_valido(&processo.unidade)
        || !id_valido(&tipo.id)
        || !texto_valido(&tipo.nome)
        || tipo.limite_bytes == 0
    {
        return Err(Falha::EntradaInvalida);
    }
    documentos::registrar_processo(banco, origem, cenario, processo).map_err(|e| match e {
        documentos::Falha::Sqlite(e) => Falha::Sqlite(e),
        documentos::Falha::EntradaInvalida => Falha::EntradaInvalida,
        documentos::Falha::ProcessoIndisponivel => Falha::ProcessoAusente,
        documentos::Falha::TipoIndisponivel => Falha::TipoAusente,
    })?;
    banco.execute(
        "INSERT INTO externo_tipo (origem, cenario, id_procedimento, id_serie, nome, limite_bytes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (origem, cenario, id_procedimento, id_serie)
         DO UPDATE SET nome=excluded.nome, limite_bytes=excluded.limite_bytes",
        params![
            origem.url_base(),
            cenario,
            processo.id,
            tipo.id,
            tipo.nome,
            numero(tipo.limite_bytes)?
        ],
    )?;
    Ok(())
}

/// Ausência de processo é distinta de processo sem documentos.
///
/// # Errors
/// Propaga erros SQLite sem convertê-los em ausência.
pub fn processo(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<Processo>, Falha> {
    documentos::ler_processo(banco, origem, cenario, id).map_err(Falha::from)
}

/// Tipos realmente disponíveis no cenário, sem inventar alternativas.
///
/// # Errors
/// Propaga a falha SQL ou recusa tamanho negativo na projeção.
pub fn tipos(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Vec<TipoExterno>, Falha> {
    let mut consulta = banco.prepare(
        "SELECT id_serie, nome, limite_bytes FROM externo_tipo
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 ORDER BY id_serie",
    )?;
    let linhas = consulta.query_map(params![origem.url_base(), cenario, id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    let mut tipos = Vec::new();
    for linha in linhas {
        let (id, nome, limite) = linha?;
        tipos.push(TipoExterno {
            id,
            nome,
            limite_bytes: u64::try_from(limite).map_err(|_| Falha::EntradaInvalida)?,
        });
    }
    Ok(tipos)
}

/// Registra exatamente o arquivo recebido, sem cadastrar documento.
///
/// # Errors
/// Recusa upload fora do processo/limite, além de preservar erro SQLite.
pub fn receber(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
    nome: &str,
    tamanho: u64,
) -> Result<Temporario, Falha> {
    if !texto_valido(nome) || tamanho == 0 {
        return Err(Falha::EntradaInvalida);
    }
    if processo(banco, origem, cenario, id_procedimento)?.is_none() {
        return Err(Falha::ProcessoAusente);
    }
    let limites = tipos(banco, origem, cenario, id_procedimento)?;
    if limites.is_empty() {
        return Err(Falha::TipoAusente);
    }
    if !limites.iter().any(|t| tamanho <= t.limite_bytes) {
        return Err(Falha::LimiteExcedido);
    }
    banco.execute(
        "INSERT INTO externo_upload (origem, cenario, id_procedimento, nome, tamanho)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            origem.url_base(),
            cenario,
            id_procedimento,
            nome,
            numero(tamanho)?
        ],
    )?;
    Ok(Temporario {
        id: banco.last_insert_rowid(),
        nome: nome.to_owned(),
        tamanho,
    })
}

/// Um cadastro definitivo consome um recibo existente uma única vez.
/// A atualização do número é atômica com a criação, sem documento fantasma.
///
/// # Errors
/// Recusa tipo, temporário ou tamanho incompatível; propaga erro SQLite.
pub fn cadastrar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
    id_serie: &str,
    id_upload: i64,
) -> Result<Documento, Falha> {
    let tipo = tipos(banco, origem, cenario, id_procedimento)?
        .into_iter()
        .find(|tipo| tipo.id == id_serie)
        .ok_or(Falha::TipoAusente)?;
    let temporario: Option<(String, i64)> = banco
        .query_row(
            "SELECT nome, tamanho FROM externo_upload
             WHERE id=?1 AND origem=?2 AND cenario=?3 AND id_procedimento=?4",
            params![id_upload, origem.url_base(), cenario, id_procedimento],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (nome, tamanho) = temporario.ok_or(Falha::UploadAusente)?;
    let tamanho = u64::try_from(tamanho).map_err(|_| Falha::EntradaInvalida)?;
    if tamanho > tipo.limite_bytes {
        return Err(Falha::LimiteExcedido);
    }
    let transacao = banco.unchecked_transaction()?;
    transacao.execute(
        "INSERT INTO externo_documento
         (origem, cenario, id_procedimento, id_serie, id_upload, numero_sei, nome, tamanho)
         VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, ?7)",
        params![
            origem.url_base(),
            cenario,
            id_procedimento,
            id_serie,
            id_upload,
            nome,
            numero(tamanho)?
        ],
    )?;
    let id = transacao.last_insert_rowid();
    let numero_sei = format!("{:09}", 50_000_000 + id);
    transacao.execute(
        "UPDATE externo_documento SET numero_sei=?1 WHERE id=?2",
        params![numero_sei, id],
    )?;
    transacao.commit()?;
    Ok(Documento {
        id,
        numero_sei,
        id_serie: id_serie.to_owned(),
        tipo: tipo.nome,
        nome,
        tamanho,
    })
}

/// Releitura independente comprova somente documentos definitivos.
///
/// # Errors
/// Propaga erros SQL sem transformar ausência em documento vazio.
pub fn documentos(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
) -> Result<Vec<Documento>, Falha> {
    let mut consulta = banco.prepare(
        "SELECT d.id, d.numero_sei, d.id_serie, t.nome, d.nome, d.tamanho
         FROM externo_documento d JOIN externo_tipo t ON
           (t.origem=d.origem AND t.cenario=d.cenario
            AND t.id_procedimento=d.id_procedimento AND t.id_serie=d.id_serie)
         WHERE d.origem=?1 AND d.cenario=?2 AND d.id_procedimento=?3 ORDER BY d.id",
    )?;
    let linhas = consulta.query_map(params![origem.url_base(), cenario, id_procedimento], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, i64>(5)?,
        ))
    })?;
    let mut documentos = Vec::new();
    for linha in linhas {
        let (id, numero_sei, id_serie, tipo, nome, tamanho) = linha?;
        documentos.push(Documento {
            id,
            numero_sei,
            id_serie,
            tipo,
            nome,
            tamanho: u64::try_from(tamanho).map_err(|_| Falha::EntradaInvalida)?,
        });
    }
    Ok(documentos)
}
