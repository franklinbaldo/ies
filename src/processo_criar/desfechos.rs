//! #245: decisão conservadora a partir dos candidatos novos na caixa Gerados
//! e das propriedades efetivamente lidas por processo NUP.
//! A ausência observada de candidatos NÃO é uma prova de não aplicação.
use super::CandidatoGerado;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessoConferido {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: Option<String>,
    pub(crate) especificacao: Option<String>,
    /// O nível lido integra a identidade do pedido; ausência não prova sucesso.
    pub(crate) nivel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Desfecho {
    Aplicada { nup: String },
    CandidatosAmbiguos { nups: Vec<String> },
    SemCorrespondenciaObservada,
}

#[derive(Debug, Clone, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroConferencia {
    #[causa(nome = "candidato_sem_leitura_verificadora")]
    LeituraAusente,
    #[causa(nome = "leitura_verificadora_repetida")]
    LeituraRepetida,
    #[causa(nome = "leitura_verificadora_de_outro_processo")]
    IdentidadeDivergente,
    #[causa(nome = "propriedades_do_candidato_nao_observadas")]
    PropriedadesNaoObservadas,
}

/// Somente leituras independentes das propriedades podem embasar o desfecho.
/// O chamador deve verificar todos os candidatos; sem isso, falha fechado.
pub(crate) fn classificar(
    candidatos: &[CandidatoGerado],
    conferidos: &[ProcessoConferido],
    tipo: &str,
    especificacao: &str,
    nivel: &str,
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
        let (Some(tipo_lido), Some(especificacao_lida), Some(nivel_lido)) =
            (&lido.tipo, &lido.especificacao, &lido.nivel)
        else {
            return Err(ErroConferencia::PropriedadesNaoObservadas);
        };
        if tipo_lido == tipo && especificacao_lida == especificacao && nivel_lido == nivel {
            correspondentes.push(candidato.nup.clone());
        }
    }
    Ok(match correspondentes.as_slice() {
        [nup] => Desfecho::Aplicada { nup: nup.clone() },
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

    fn prova(id: &str, nup: &str, tipo: Option<&str>, esp: Option<&str>) -> ProcessoConferido {
        ProcessoConferido {
            id: id.into(),
            nup: nup.into(),
            tipo: tipo.map(str::to_owned),
            especificacao: esp.map(str::to_owned),
            nivel: Some("publico".into()),
        }
    }

    #[test]
    fn unica_prova_com_tipo_e_especificacao_exatos_conclui_aplicada() {
        let candidatos = vec![candidato("17", "00000.000017/2026-17")];
        let conferidos = vec![prova(
            "17",
            "00000.000017/2026-17",
            Some(TIPO),
            Some(ESPECIFICACAO),
        )];
        assert_eq!(
            classificar(&candidatos, &conferidos, TIPO, ESPECIFICACAO, "publico"),
            Ok(Desfecho::Aplicada {
                nup: "00000.000017/2026-17".into()
            })
        );
    }

    #[test]
    fn dois_processos_conferidos_com_mesmo_tipo_e_especificacao_sao_ambiguos() {
        let candidatos = vec![candidato("17", "NUP-17"), candidato("18", "NUP-18")];
        let conferidos = vec![
            prova("18", "NUP-18", Some(TIPO), Some(ESPECIFICACAO)),
            prova("17", "NUP-17", Some(TIPO), Some(ESPECIFICACAO)),
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
            prova("17", "NUP-17", Some("Outro tipo"), Some(ESPECIFICACAO)),
            prova("18", "NUP-18", Some(TIPO), Some("Outra especificacao")),
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
            prova("17", "NUP-17", Some(TIPO), Some(ESPECIFICACAO)),
            prova("17", "NUP-17", Some(TIPO), Some(ESPECIFICACAO)),
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
            prova("18", "NUP-17", Some(TIPO), Some(ESPECIFICACAO)),
            prova("17", "NUP-18", Some(TIPO), Some(ESPECIFICACAO)),
        ] {
            assert_eq!(
                classificar(&candidatos, &[prova_errada], TIPO, ESPECIFICACAO, "publico"),
                Err(ErroConferencia::IdentidadeDivergente)
            );
        }
    }

    #[test]
    fn nivel_divergente_ou_nao_observado_nao_comprova_criacao() {
        let candidatos = vec![candidato("17", "NUP-17")];
        let mut prova_restrita = prova("17", "NUP-17", Some(TIPO), Some(ESPECIFICACAO));
        prova_restrita.nivel = Some("restrito".into());
        assert_eq!(
            classificar(
                &candidatos,
                &[prova_restrita.clone()],
                TIPO,
                ESPECIFICACAO,
                "publico"
            ),
            Ok(Desfecho::SemCorrespondenciaObservada)
        );
        prova_restrita.nivel = None;
        assert_eq!(
            classificar(
                &candidatos,
                &[prova_restrita],
                TIPO,
                ESPECIFICACAO,
                "publico"
            ),
            Err(ErroConferencia::PropriedadesNaoObservadas)
        );
    }

    #[test]
    fn campo_nao_observado_impede_conclusao_positiva_ou_negativa() {
        let candidatos = vec![candidato("17", "NUP-17")];
        for prova_incompleta in [
            prova("17", "NUP-17", None, Some(ESPECIFICACAO)),
            prova("17", "NUP-17", Some(TIPO), None),
        ] {
            assert_eq!(
                classificar(
                    &candidatos,
                    &[prova_incompleta],
                    TIPO,
                    ESPECIFICACAO,
                    "publico"
                ),
                Err(ErroConferencia::PropriedadesNaoObservadas)
            );
        }
    }
}
