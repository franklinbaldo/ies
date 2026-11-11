//! Estados e transições de processos no banco descartável do anti-IES.
//! A hipótese é sintética, não uma afirmação sobre o protocolo SEI real.
use crate::banco_cassettes::{ControleResumido, Linha};
use crate::instalacao::Instalacao;
use crate::observacao::{FaixaObservada, TabelaObservada};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Destino {
    pub id: String,
    pub sigla: String,
    pub descricao: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessoSimulado {
    pub id: String,
    pub numero: String,
    pub unidade_origem: String,
    pub aberto_em: Vec<String>,
    pub anotacao_origem: Option<String>,
    pub destinos: Vec<Destino>,
}

impl ProcessoSimulado {
    /// A inclusão documental ainda é oferecida somente na origem aberta.
    /// A unidade de destino pode tramitar, mas não criar documento neste cenário.
    #[must_use]
    pub fn pode_incluir_documento(&self, unidade_atual: &str) -> bool {
        unidade_atual == self.unidade_origem
            && self
                .aberto_em
                .iter()
                .any(|unidade| unidade == unidade_atual)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PedidoEnvio<'a> {
    pub id: &'a str,
    pub id_destino: &'a str,
    pub manter_aberto: bool,
    pub remover_anotacoes: bool,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroProcessos {
    Sqlite(#[from] rusqlite::Error),
    DadoInvalido,
    ProcessoNaoEncontrado,
    OrigemNaoAberta,
    DestinoNaoOferecido,
    DestinoJaAberto,
    AnotacaoNaoObservada,
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())
}

fn validar(processo: &ProcessoSimulado) -> Result<(), ErroProcessos> {
    if !id_valido(&processo.id)
        || processo.numero.is_empty()
        || processo.unidade_origem.is_empty()
        || processo.aberto_em.iter().any(String::is_empty)
        || processo
            .destinos
            .iter()
            .any(|d| !id_valido(&d.id) || d.sigla.is_empty() || d.descricao.is_empty())
    {
        return Err(ErroProcessos::DadoInvalido);
    }
    for (i, unidade) in processo.aberto_em.iter().enumerate() {
        if processo
            .aberto_em
            .iter()
            .take(i)
            .any(|anterior| anterior == unidade)
        {
            return Err(ErroProcessos::DadoInvalido);
        }
    }
    for (i, destino) in processo.destinos.iter().enumerate() {
        if processo
            .destinos
            .iter()
            .take(i)
            .any(|anterior| anterior.id == destino.id || anterior.sigla == destino.sigla)
        {
            return Err(ErroProcessos::DadoInvalido);
        }
    }
    Ok(())
}

/// Reconstrói um cenário com suas relações numa transação indivisível.
///
/// # Errors
/// Rejeita identidade duplicada/inválida ou preserva o erro SQLite.
pub fn guardar(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    processo: &ProcessoSimulado,
) -> Result<(), ErroProcessos> {
    validar(processo)?;
    if cenario.is_empty() {
        return Err(ErroProcessos::DadoInvalido);
    }
    let origem = instalacao.url_base();
    let tx = banco.transaction()?;
    // Identidade canônica de toda a simulação: documentos e tramitação
    // consultam o mesmo NUP e a mesma unidade, nunca cópias divergentes.
    tx.execute(
        "INSERT INTO sim_processo
         (origem, cenario, id_procedimento, nup, unidade)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(origem, cenario, id_procedimento) DO UPDATE SET
         nup=excluded.nup, unidade=excluded.unidade",
        params![
            origem,
            cenario,
            processo.id,
            processo.numero,
            processo.unidade_origem
        ],
    )?;
    tx.execute(
        "INSERT INTO processo_simulado
         (origem, cenario, id_procedimento, anotacao_origem)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(origem, cenario, id_procedimento) DO UPDATE SET
         anotacao_origem=excluded.anotacao_origem",
        params![origem, cenario, processo.id, processo.anotacao_origem],
    )?;
    tx.execute(
        "DELETE FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, processo.id],
    )?;
    tx.execute(
        "DELETE FROM processo_destino
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, processo.id],
    )?;
    for (posicao, unidade) in processo.aberto_em.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| ErroProcessos::DadoInvalido)?;
        tx.execute(
            "INSERT INTO processo_abertura
             (origem, cenario, id_procedimento, posicao, unidade)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![origem, cenario, processo.id, posicao, unidade],
        )?;
    }
    for destino in &processo.destinos {
        tx.execute(
            "INSERT INTO processo_destino
             (origem, cenario, id_procedimento, id_unidade, sigla, descricao)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                origem,
                cenario,
                processo.id,
                destino.id,
                destino.sigla,
                destino.descricao
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Consulta a mesma fotografia que o servidor emprega em GET e POST.
///
/// # Errors
/// Propaga erros SQLite e estado estruturalmente inválido.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<ProcessoSimulado>, ErroProcessos> {
    let origem = instalacao.url_base();
    let cabecalho: Option<(String, String, Option<String>)> = banco
        .query_row(
            "SELECT base.nup, base.unidade, envio.anotacao_origem
             FROM processo_simulado envio JOIN sim_processo base
             ON base.origem=envio.origem AND base.cenario=envio.cenario
             AND base.id_procedimento=envio.id_procedimento
             WHERE envio.origem=?1 AND envio.cenario=?2 AND envio.id_procedimento=?3",
            params![origem, cenario, id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((numero, unidade_origem, anotacao_origem)) = cabecalho else {
        return Ok(None);
    };
    let mut aberturas = banco.prepare(
        "SELECT unidade FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3
         ORDER BY posicao",
    )?;
    let aberto_em = aberturas
        .query_map(params![origem, cenario, id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    let mut consulta = banco.prepare(
        "SELECT id_unidade, sigla, descricao FROM processo_destino
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 ORDER BY id_unidade",
    )?;
    let destinos = consulta
        .query_map(params![origem, cenario, id], |r| {
            Ok(Destino {
                id: r.get(0)?,
                sigla: r.get(1)?,
                descricao: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<Destino>>>()?;
    let processo = ProcessoSimulado {
        id: id.to_owned(),
        numero,
        unidade_origem,
        aberto_em,
        anotacao_origem,
        destinos,
    };
    validar(&processo)?;
    Ok(Some(processo))
}

/// Tramita uma vez; dados inválidos não produzem escrita parcial.
///
/// # Errors
/// Distingue processo ausente, origem já fechada, destino não oferecido,
/// destino já aberto, anotação não observada e erro SQLite real.
pub fn enviar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    pedido: PedidoEnvio<'_>,
) -> Result<ProcessoSimulado, ErroProcessos> {
    enviar_com_unidade(banco, instalacao, cenario, pedido, None)
}

/// Envia a partir da unidade ativa no Controle R, não necessariamente a
/// unidade na qual o processo foi originalmente cadastrado.
///
/// # Errors
/// Recusa unidade fechada e propaga a causa SQLite ou de validação.
pub fn enviar_da_unidade(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    pedido: PedidoEnvio<'_>,
    unidade: &str,
) -> Result<ProcessoSimulado, ErroProcessos> {
    enviar_com_unidade(banco, instalacao, cenario, pedido, Some(unidade))
}

fn enviar_com_unidade(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    pedido: PedidoEnvio<'_>,
    unidade: Option<&str>,
) -> Result<ProcessoSimulado, ErroProcessos> {
    let tx = banco.unchecked_transaction()?;
    let mut processo =
        ler(&tx, instalacao, cenario, pedido.id)?.ok_or(ErroProcessos::ProcessoNaoEncontrado)?;
    let unidade_atual = unidade.unwrap_or(&processo.unidade_origem).to_owned();
    if !processo.aberto_em.contains(&unidade_atual) {
        return Err(ErroProcessos::OrigemNaoAberta);
    }
    let destino = processo
        .destinos
        .iter()
        .find(|d| d.id == pedido.id_destino)
        .ok_or(ErroProcessos::DestinoNaoOferecido)?;
    if processo.aberto_em.contains(&destino.sigla) {
        return Err(ErroProcessos::DestinoJaAberto);
    }
    if pedido.remover_anotacoes {
        if unidade_atual != processo.unidade_origem {
            return Err(ErroProcessos::AnotacaoNaoObservada);
        }
        let anotacao = processo
            .anotacao_origem
            .as_mut()
            .ok_or(ErroProcessos::AnotacaoNaoObservada)?;
        anotacao.clear();
    }
    let sigla = destino.sigla.clone();
    if !pedido.manter_aberto {
        processo
            .aberto_em
            .retain(|unidade| unidade != &unidade_atual);
    }
    processo.aberto_em.push(sigla);
    let origem = instalacao.url_base();
    tx.execute(
        "UPDATE processo_simulado SET anotacao_origem=?4
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, pedido.id, processo.anotacao_origem],
    )?;
    tx.execute(
        "DELETE FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, pedido.id],
    )?;
    for (i, unidade) in processo.aberto_em.iter().enumerate() {
        let i = i64::try_from(i).map_err(|_| ErroProcessos::DadoInvalido)?;
        tx.execute(
            "INSERT INTO processo_abertura
             (origem, cenario, id_procedimento, posicao, unidade)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![origem, cenario, pedido.id, i, unidade],
        )?;
    }
    tx.commit()?;
    Ok(processo)
}

/// Simula a alternativa de cadastro da anotação antes do envio.
///
/// # Errors
/// Preserva ausência da observação e erro SQLite.
pub fn limpar_anotacao(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<(), ErroProcessos> {
    registrar_anotacao(banco, instalacao, cenario, id, "")
}

/// Registra texto livre, preservando a diferença entre dado desconhecido
/// e uma anotação explicitamente observada como vazia. Um único UPDATE
/// torna o texto visível para leituras seguintes no mesmo SQLite.
///
/// # Errors
/// Recusa processo sem observação de anotação ou origem fechada.
pub fn registrar_anotacao(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    texto: &str,
) -> Result<(), ErroProcessos> {
    let processo =
        ler(banco, instalacao, cenario, id)?.ok_or(ErroProcessos::ProcessoNaoEncontrado)?;
    if !processo.aberto_em.contains(&processo.unidade_origem) {
        return Err(ErroProcessos::OrigemNaoAberta);
    }
    if processo.anotacao_origem.is_none() {
        return Err(ErroProcessos::AnotacaoNaoObservada);
    }
    banco.execute(
        "UPDATE processo_simulado SET anotacao_origem=?4
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![instalacao.url_base(), cenario, id, texto],
    )?;
    Ok(())
}

/// Somente uma página integral autoriza atualizar contagem e presença de linhas.
/// Uma página parcial continua tendo exatamente a cobertura que foi observada.
fn pagina_inteira(tabela: &TabelaObservada<Linha>) -> bool {
    if tabela.total_declarado != Some(tabela.linhas.len()) {
        return false;
    }
    match (tabela.faixa_observada, tabela.linhas.len()) {
        (None, 0) => true,
        (Some(faixa), tamanho) if tamanho > 0 => faixa.inicio() == 1 && faixa.fim() == tamanho,
        _ => false,
    }
}

fn tabela_atualizada(linhas: Vec<Linha>) -> Result<TabelaObservada<Linha>, ErroProcessos> {
    let tamanho = linhas.len();
    TabelaObservada::nova(Some(tamanho), FaixaObservada::nova(1, tamanho), linhas)
        .map_err(|_| ErroProcessos::DadoInvalido)
}

/// Projeta as transições do SQLite no Controle R da unidade ativa.
///
/// O snapshot conserva as linhas alheias à simulação. Apenas processos com
/// estado explícito em `processo_simulado` mudam de presença, e somente quando
/// as tabelas do Controle R representam páginas integrais.
///
/// # Errors
/// Propaga a causa SQLite sem classificar pelo texto.
pub fn controle_da_unidade(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    mut controle: ControleResumido,
) -> Result<ControleResumido, ErroProcessos> {
    if !pagina_inteira(&controle.recebidos) || !pagina_inteira(&controle.gerados) {
        return Ok(controle);
    }
    let mut consulta = banco.prepare(
        "SELECT base.id_procedimento, base.nup,
                EXISTS(SELECT 1 FROM processo_abertura aberta
                       WHERE aberta.origem=base.origem
                         AND aberta.cenario=base.cenario
                         AND aberta.id_procedimento=base.id_procedimento
                         AND aberta.unidade=?3)
         FROM sim_processo base
         JOIN processo_simulado sim
           ON sim.origem=base.origem AND sim.cenario=base.cenario
          AND sim.id_procedimento=base.id_procedimento
         WHERE base.origem=?1 AND base.cenario=?2
         ORDER BY base.id_procedimento",
    )?;
    let estados = consulta
        .query_map(
            params![instalacao.url_base(), cenario, controle.unidade.as_str()],
            |linha| {
                Ok((
                    linha.get::<_, String>(0)?,
                    linha.get::<_, String>(1)?,
                    linha.get::<_, i64>(2)? != 0,
                ))
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut mudou = false;
    for (id, numero, aberta) in estados {
        if !aberta {
            let recebidos = controle.recebidos.linhas.len();
            let gerados = controle.gerados.linhas.len();
            controle.recebidos.linhas.retain(|linha| linha.id != id);
            controle.gerados.linhas.retain(|linha| linha.id != id);
            mudou |= recebidos != controle.recebidos.linhas.len()
                || gerados != controle.gerados.linhas.len();
        } else if !controle.recebidos.linhas.iter().any(|linha| linha.id == id)
            && !controle.gerados.linhas.iter().any(|linha| linha.id == id)
        {
            controle.recebidos.linhas.push(Linha {
                id,
                numero,
                visualizado: false,
                sigiloso: false,
            });
            mudou = true;
        }
    }
    if mudou {
        controle.recebidos = tabela_atualizada(controle.recebidos.linhas)?;
        controle.gerados = tabela_atualizada(controle.gerados.linhas)?;
    }
    Ok(controle)
}

/// Lista os destinos oferecidos no cenário, compartilhados entre processos.
///
/// Um cenário inexistente não equivale a lista vazia. Uma lista vazia,
/// em contraste, pode representar um processo conhecido sem destinos.
///
/// # Errors
/// Preserva erros da leitura SQLite.
pub fn destinos_do_cenario(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<Option<Vec<Destino>>, ErroProcessos> {
    let origem = instalacao.url_base();
    let observado: i64 = banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_simulado WHERE origem=?1 AND cenario=?2)",
        params![origem, cenario],
        |linha| linha.get(0),
    )?;
    if observado == 0 {
        return Ok(None);
    }
    let mut consulta = banco.prepare(
        "SELECT DISTINCT id_unidade, sigla, descricao FROM processo_destino
         WHERE origem=?1 AND cenario=?2 ORDER BY sigla, id_unidade",
    )?;
    let destinos = consulta
        .query_map(params![origem, cenario], |linha| {
            Ok(Destino {
                id: linha.get(0)?,
                sigla: linha.get(1)?,
                descricao: linha.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(destinos))
}
