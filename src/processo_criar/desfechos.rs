//! #245: decisão conservadora a partir dos candidatos novos na caixa Gerados
//! e das propriedades efetivamente lidas por processo NUP.
//! A ausência observada de candidatos NÃO é uma prova de não aplicação.
use super::CandidatoGerado;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessoConferido {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: String,
    pub(crate) especificacao: String,
    /// O nível lido integra a identidade do pedido.
    pub(crate) nivel: String,
    /// Interessados publicados na tela de propriedades.
    pub(crate) interessados: Vec<String>,
}

impl ProcessoConferido {
    /// Uma leitura só prova o pedido com o tipo publicado: sem ele, a
    /// conferência falha fechada antes de qualquer comparação.
    pub(crate) fn novo(
        id: &str,
        nup: &str,
        tipo: Option<String>,
        especificacao: String,
        nivel: String,
        interessados: Vec<String>,
    ) -> Result<Self, ErroConferencia> {
        Ok(Self {
            id: id.to_owned(),
            nup: nup.to_owned(),
            tipo: tipo.ok_or(ErroConferencia::TipoDoProcessoNaoMarcado)?,
            especificacao,
            nivel,
            interessados,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Desfecho {
    /// Uma correspondencia de propriedades: NAO prova qual POST a criou.
    CoincidenciaUnica {
        nup: String,
    },
    CandidatosAmbiguos {
        nups: Vec<String>,
    },
    SemCorrespondenciaObservada,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroConferencia {
    LeituraAusente,
    LeituraRepetida,
    IdentidadeDivergente,
    /// A tela de propriedades não marca nenhum tipo de processo.
    TipoDoProcessoNaoMarcado,
}

/// Quando há interessados explícitos, exigimos o mesmo multiconjunto
/// publicado no formulário de propriedades. Ordem de apresentação não é
/// identidade; multiplicidade e texto observado são.
pub(crate) fn interessados_conferem(solicitados: &[String], observados: &[String]) -> bool {
    if solicitados.is_empty() {
        return true; // nenhum interessado imposto pelo usuário
    }
    if solicitados.len() != observados.len() {
        return false;
    }
    let mut solicitados = solicitados.to_vec();
    let mut observados = observados.to_vec();
    solicitados.sort_unstable();
    observados.sort_unstable();
    solicitados == observados
}

/// Somente leituras independentes das propriedades podem embasar o desfecho.
/// O chamador deve verificar todos os candidatos; sem isso, falha fechado.
pub(crate) fn classificar(
    candidatos: &[CandidatoGerado],
    conferidos: &[ProcessoConferido],
    tipo: &str,
    especificacao: &str,
    nivel: &str,
    interessados_solicitados: &[String],
) -> Result<Desfecho, ErroConferencia> {
    use std::collections::BTreeMap;

    // Uma prova deve vir de uma leitura distinta, para um dos pares
    // (id, NUP) da fotografia; jamais se aceita uma prova solta ou
    // repetida, nem uma leitura incompleta como divergência de conteúdo.
    let mut por_id = BTreeMap::new();
    for conferido in conferidos {
        if por_id.insert(conferido.id.as_str(), conferido).is_some() {
            return Err(ErroConferencia::LeituraRepetida);
        }
        if !candidatos
            .iter()
            .any(|c| c.id == conferido.id && c.nup == conferido.nup)
        {
            return Err(ErroConferencia::IdentidadeDivergente);
        }
    }
    let mut correspondentes = Vec::new();
    for candidato in candidatos {
        let lido = por_id
            .get(candidato.id.as_str())
            .ok_or(ErroConferencia::LeituraAusente)?;
        if lido.tipo == tipo
            && lido.especificacao == especificacao
            && lido.nivel == nivel
            && interessados_conferem(interessados_solicitados, &lido.interessados)
        {
            correspondentes.push(candidato.nup.clone());
        }
    }
    Ok(match correspondentes.as_slice() {
        [nup] => Desfecho::CoincidenciaUnica { nup: nup.clone() },
        [] => Desfecho::SemCorrespondenciaObservada,
        _ => Desfecho::CandidatosAmbiguos {
            nups: correspondentes,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIPO: &str = "Teste: processo";
    const ESPECIFICACAO: &str = "Ensaio issue 245";

    fn candidato(id: &str, nup: &str) -> CandidatoGerado {
        CandidatoGerado {
            id: id.into(),
            nup: nup.into(),
        }
    }

    fn prova(id: &str, nup: &str, tipo: &str, esp: &str) -> ProcessoConferido {
        ProcessoConferido {
            id: id.into(),
            nup: nup.into(),
            tipo: tipo.into(),
            especificacao: esp.into(),
            nivel: "publico".into(),
            interessados: vec!["Pessoa Sintetica A".into()],
        }
    }

    // O classificador em produção recebe também o pedido de interessados.
    // Os testes legados aqui não solicitam interessado explícito.
    fn classificar(
        candidatos: &[CandidatoGerado],
        conferidos: &[ProcessoConferido],
        tipo: &str,
        especificacao: &str,
        nivel: &str,
    ) -> Result<Desfecho, ErroConferencia> {
        super::classificar(candidatos, conferidos, tipo, especificacao, nivel, &[])
    }

    #[test]
    fn interessado_explicito_divergente_nao_comprova_aplicacao_concorrente() {
        let candidatos = vec![candidato("17", "NUP-17")];
        let lido = prova("17", "NUP-17", TIPO, ESPECIFICACAO);
        // O processo concorrente tem o mesmo tipo, especificacao e nivel,
        // mas outro interessado. O POST original pode ter falhado.
        let desejado = vec!["Pessoa Sintetica B".to_owned()];
        assert_eq!(
            super::classificar(
                &candidatos,
                &[lido],
                TIPO,
                ESPECIFICACAO,
                "publico",
                &desejado
            ),
            Ok(Desfecho::SemCorrespondenciaObservada)
        );
    }

    #[test]
    fn interessado_explicito_igual_em_outra_ordem_pode_ser_comprovado() {
        let requeridos = vec!["Pessoa B".to_owned(), "Pessoa A".to_owned()];
        let lidos = vec!["Pessoa A".to_owned(), "Pessoa B".to_owned()];
        assert!(interessados_conferem(&requeridos, &lidos));
        assert!(!interessados_conferem(
            &["Pessoa A".to_owned(), "Pessoa A".to_owned()],
            &lidos
        ));
        assert!(interessados_conferem(&[], &lidos));
    }

    #[test]
    fn unica_prova_com_tipo_e_especificacao_exatos_conclui_aplicada() {
        let candidatos = vec![candidato("17", "00000.000017/2026-17")];
        let conferidos = vec![prova("17", "00000.000017/2026-17", TIPO, ESPECIFICACAO)];
        assert_eq!(
            classificar(&candidatos, &conferidos, TIPO, ESPECIFICACAO, "publico"),
            Ok(Desfecho::CoincidenciaUnica {
                nup: "00000.000017/2026-17".into()
            })
        );
    }

    #[test]
    fn dois_processos_conferidos_com_mesmo_tipo_e_especificacao_sao_ambiguos() {
        let candidatos = vec![candidato("17", "NUP-17"), candidato("18", "NUP-18")];
        let conferidos = vec![
            prova("18", "NUP-18", TIPO, ESPECIFICACAO),
            prova("17", "NUP-17", TIPO, ESPECIFICACAO),
        ];
        assert_eq!(
            classificar(&candidatos, &conferidos, TIPO, ESPECIFICACAO, "publico"),
            Ok(Desfecho::CandidatosAmbiguos {
                nups: vec!["NUP-17".into(), "NUP-18".into()]
            })
        );
    }

    #[test]
    fn tipo_ou_especificacao_divergentes_nao_sao_aplicacao() {
        let candidatos = vec![candidato("17", "NUP-17"), candidato("18", "NUP-18")];
        let conferidos = vec![
            prova("17", "NUP-17", "Outro tipo", ESPECIFICACAO),
            prova("18", "NUP-18", TIPO, "Outra especificacao"),
        ];
        assert_eq!(
            classificar(&candidatos, &conferidos, TIPO, ESPECIFICACAO, "publico"),
            Ok(Desfecho::SemCorrespondenciaObservada)
        );
    }

    #[test]
    fn nenhum_candidato_na_fotografia_nao_prova_nao_aplicacao() {
        assert_eq!(
            classificar(&[], &[], TIPO, ESPECIFICACAO, "publico"),
            Ok(Desfecho::SemCorrespondenciaObservada)
        );
    }

    #[test]
    fn candidato_sem_leitura_nao_pode_ser_ignorado() {
        let candidatos = vec![candidato("17", "NUP-17")];
        assert_eq!(
            classificar(&candidatos, &[], TIPO, ESPECIFICACAO, "publico"),
            Err(ErroConferencia::LeituraAusente)
        );
    }

    #[test]
    fn prova_duplicada_nao_atesta_unicidade() {
        let candidatos = vec![candidato("17", "NUP-17")];
        let conferidos = vec![
            prova("17", "NUP-17", TIPO, ESPECIFICACAO),
            prova("17", "NUP-17", TIPO, ESPECIFICACAO),
        ];
        assert_eq!(
            classificar(&candidatos, &conferidos, TIPO, ESPECIFICACAO, "publico"),
            Err(ErroConferencia::LeituraRepetida)
        );
    }

    #[test]
    fn prova_com_id_ou_nup_divergente_nao_identifica_candidato() {
        let candidatos = vec![candidato("17", "NUP-17")];
        for prova_errada in [
            prova("18", "NUP-17", TIPO, ESPECIFICACAO),
            prova("17", "NUP-18", TIPO, ESPECIFICACAO),
        ] {
            assert_eq!(
                classificar(&candidatos, &[prova_errada], TIPO, ESPECIFICACAO, "publico"),
                Err(ErroConferencia::IdentidadeDivergente)
            );
        }
    }

    #[test]
    fn nivel_divergente_nao_comprova_criacao() {
        let candidatos = vec![candidato("17", "NUP-17")];
        let mut prova_restrita = prova("17", "NUP-17", TIPO, ESPECIFICACAO);
        prova_restrita.nivel = "restrito".into();
        assert_eq!(
            classificar(
                &candidatos,
                &[prova_restrita],
                TIPO,
                ESPECIFICACAO,
                "publico"
            ),
            Ok(Desfecho::SemCorrespondenciaObservada)
        );
    }

    #[test]
    fn tipo_nao_marcado_na_tela_impede_a_leitura_da_conferencia() {
        assert_eq!(
            ProcessoConferido::novo(
                "17",
                "NUP-17",
                None,
                ESPECIFICACAO.into(),
                "publico".into(),
                Vec::new()
            ),
            Err(ErroConferencia::TipoDoProcessoNaoMarcado)
        );
        assert!(
            ProcessoConferido::novo(
                "17",
                "NUP-17",
                Some(TIPO.into()),
                ESPECIFICACAO.into(),
                "publico".into(),
                Vec::new()
            )
            .is_ok()
        );
    }
}
