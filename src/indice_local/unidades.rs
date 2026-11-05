//! Lista integral da lupa do SEI, sem URLs ou credenciais no banco.
//! A tabela 'unidade' é contrato público somente para SELECT (#244).
use super::IndiceLocal;
use rusqlite::params;
use std::collections::HashSet;

/// Unidade observada na lupa, com identidade opaca.
// A integração Web/CLI da lista ainda está em desenvolvimento na #244.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "integração Web/CLI pendente na #244")
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnidadeGuardada {
    pub id: String,
    pub sigla: String,
    pub descricao: String,
    pub orgao: String,
}

/// Snapshot confirmado como integral; não confundir uma lista ausente
/// com uma lista legitimamente vazia.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "integração Web/CLI pendente na #244")
)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ListaGuardada {
    pub unidades: Vec<UnidadeGuardada>,
    pub lida_em: f64,
    pub total: usize,
}

impl IndiceLocal {
    /// Substitui uma lista inteira numa transação: sem gravação parcial.
    /// O chamador deve conferir páginas, total declarado, identidade da
    /// instalação, IDs duplicados e forma HTML ANTES desta chamada.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "integração Web/CLI pendente na #244")
    )]
    pub(crate) fn guardar_lista_de_unidades(
        &self,
        instalacao: &str,
        unidade_ativa: &str,
        unidades: &[UnidadeGuardada],
        total_declarado: usize,
    ) -> bool {
        let ids: HashSet<&str> = unidades.iter().map(|u| u.id.as_str()).collect();
        if instalacao.is_empty()
            || unidade_ativa.is_empty()
            || total_declarado != unidades.len()
            || ids.len() != unidades.len()
            || unidades
                .iter()
                .any(|u| u.id.is_empty() || u.sigla.is_empty())
        {
            return false;
        }
        let Ok(total_convertido) = i64::try_from(total_declarado) else {
            return false;
        };
        self.executar(false, |banco, agora| {
            banco.execute(
                "DELETE FROM unidade WHERE instalacao=?1 AND unidade_ativa=?2",
                params![instalacao, unidade_ativa],
            )?;
            banco.execute(
                "INSERT INTO lista_unidade(instalacao, unidade_ativa, lida_em, total)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(instalacao, unidade_ativa) DO UPDATE
                 SET lida_em=excluded.lida_em, total=excluded.total",
                params![instalacao, unidade_ativa, agora, total_convertido],
            )?;
            let mut inserir = banco.prepare(
                "INSERT INTO unidade(instalacao, unidade_ativa, id, sigla, descricao, orgao, lida_em)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for u in unidades {
                inserir.execute(params![
                    instalacao, unidade_ativa, u.id, u.sigla, u.descricao, u.orgao, agora
                ])?;
            }
            Ok(true)
        })
    }

    /// Sem requisição ao SEI. None significa lista ausente ou índice desligado.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "integração Web/CLI pendente na #244")
    )]
    pub(crate) fn lista_de_unidades(
        &self,
        instalacao: &str,
        unidade_ativa: &str,
    ) -> Option<ListaGuardada> {
        self.executar(None, |banco, _| {
            let mut cabecalho = banco.prepare(
                "SELECT lida_em, total FROM lista_unidade
                 WHERE instalacao=?1 AND unidade_ativa=?2",
            )?;
            let mut linhas = cabecalho.query(params![instalacao, unidade_ativa])?;
            let Some(linha) = linhas.next()? else {
                return Ok(None);
            };
            let lida_em: f64 = linha.get(0)?;
            let total: i64 = linha.get(1)?;
            let mut consulta = banco.prepare(
                "SELECT id, sigla, descricao, orgao FROM unidade
                 WHERE instalacao=?1 AND unidade_ativa=?2 ORDER BY sigla, id",
            )?;
            let unidades: Vec<UnidadeGuardada> = consulta
                .query_map(params![instalacao, unidade_ativa], |l| {
                    Ok(UnidadeGuardada {
                        id: l.get(0)?,
                        sigla: l.get(1)?,
                        descricao: l.get(2)?,
                        orgao: l.get(3)?,
                    })
                })?
                .collect::<rusqlite::Result<_>>()?;
            let total = usize::try_from(total).map_err(|_| rusqlite::Error::InvalidQuery)?;
            if total != unidades.len() {
                return Err(rusqlite::Error::InvalidQuery);
            }
            Ok(Some(ListaGuardada {
                unidades,
                lida_em,
                total,
            }))
        })
    }

    /// Filtra somente o snapshot, por sigla ou descrição, com sigla exata
    /// priorizada. Nenhuma requisição e nenhuma alteração no banco.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "integração Web/CLI pendente na #244")
    )]
    pub(crate) fn pesquisar_unidades_guardadas(
        &self,
        instalacao: &str,
        unidade_ativa: &str,
        termo: &str,
    ) -> Option<ListaGuardada> {
        let mut lista = self.lista_de_unidades(instalacao, unidade_ativa)?;
        let termo = chave(termo);
        lista
            .unidades
            .retain(|u| chave(&u.sigla).contains(&termo) || chave(&u.descricao).contains(&termo));
        lista
            .unidades
            .sort_by_key(|u| (chave(&u.sigla) != termo, chave(&u.sigla), u.id.clone()));
        lista.total = lista.unidades.len();
        Some(lista)
    }
}

/// Normalização de pesquisa local sem expressões regulares; preserva '/'.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "integração Web/CLI pendente na #244")
)]
fn chave(texto: &str) -> String {
    texto
        .trim()
        .chars()
        .flat_map(char::to_lowercase)
        // A lupa pode devolver acentos decompostos (Unicode NFD).
        .filter(|c| !('\u{0300}'..='\u{036f}').contains(c))
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn unidade(id: &str, sigla: &str, descricao: &str) -> UnidadeGuardada {
        UnidadeGuardada {
            id: id.to_owned(),
            sigla: sigla.to_owned(),
            descricao: descricao.to_owned(),
            orgao: "Orgão".to_owned(),
        }
    }

    #[test]
    fn normalizacao_aceita_acentos_precompostos_e_decompostos() {
        assert_eq!(chave("SÃO/CDE"), chave("Sa\u{0303}o/CDE"));
        assert_eq!(chave("DESCRIÇÃO"), chave("Descric\u{0327}a\u{0303}o"));
    }

    #[test]
    fn snapshot_inteiro_e_consulta_offline() {
        let pasta = std::env::temp_dir().join(format!(
            "ies-244-{}-{}",
            std::process::id(),
            std::thread::current()
                .name()
                .unwrap_or("test")
                .replace(':', "-")
        ));
        std::fs::create_dir_all(&pasta).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&pasta, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let caminho: PathBuf = pasta.join("ies.sqlite3");
        let indice = IndiceLocal::em(caminho);
        let lista = [
            unidade("1", "SEAD/CDE-09516", "Administração"),
            unidade("2", "RETOMADA/CDE-21385", "Descrição"),
        ];
        assert_eq!(indice.lista_de_unidades("https://sei.exemplo", "7"), None);
        assert!(indice.guardar_lista_de_unidades("https://sei.exemplo", "7", &lista, 2));
        let achadas = indice
            .pesquisar_unidades_guardadas("https://sei.exemplo", "7", "descricão")
            .unwrap();
        assert_eq!(achadas.total, 1);
        assert_eq!(achadas.unidades[0].id, "2");
        assert_eq!(indice.lista_de_unidades("https://sei.exemplo", "8"), None);
        // Página incompleta e IDs repetidos não removem o snapshot anterior.
        assert!(!indice.guardar_lista_de_unidades("https://sei.exemplo", "7", &lista[..1], 2));
        assert!(!indice.guardar_lista_de_unidades(
            "https://sei.exemplo",
            "7",
            &[lista[0].clone(), lista[0].clone()],
            2
        ));
        assert_eq!(
            indice
                .lista_de_unidades("https://sei.exemplo", "7")
                .unwrap()
                .total,
            2
        );
        assert!(indice.guardar_lista_de_unidades("https://sei.exemplo", "7", &[], 0));
        assert_eq!(
            indice
                .lista_de_unidades("https://sei.exemplo", "7")
                .unwrap()
                .total,
            0
        );
    }
}
