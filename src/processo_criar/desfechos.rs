//! #245: decisão conservadora a partir dos candidatos novos na caixa Gerados
//! e das propriedades efetivamente lidas por processo NUP.
//! A ausência observada de candidatos NÃO é uma prova de não aplicação.
use super::CandidatoGerado;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessoConferido {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) id_tipo: String,
    pub(crate) tipo: String,
    pub(crate) id_hipotese_legal: Option<String>,
    pub(crate) especificacao: String,
    /// O nível lido integra a identidade do pedido.
    pub(crate) nivel: crate::dominio::NivelDeAcesso,
    /// Interessados publicados na tela de propriedades.
    pub(crate) interessados: Vec<String>,
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

impl ProcessoConferido {
    pub(crate) fn corresponde(
        &self,
        tipo: (&str, &str),
        hipotese: Option<&str>,
        especificacao: &str,
        nivel: crate::dominio::NivelDeCriacao,
        interessados: &[String],
    ) -> bool {
        self.id_tipo == tipo.0
            && self.tipo == tipo.1
            && self.id_hipotese_legal.as_deref() == hipotese
            && self.especificacao == especificacao
            && self.nivel == nivel.como_acesso()
            && interessados_conferem(interessados, &self.interessados)
    }
}

/// Somente leituras independentes das propriedades podem embasar o desfecho.
/// O chamador deve verificar todos os candidatos; sem isso, falha fechado.
pub(crate) fn classificar(
    candidatos: &[CandidatoGerado],
    conferidos: &[ProcessoConferido],
    tipo: (&str, &str),
    hipotese: Option<&str>,
    especificacao: &str,
    nivel: crate::dominio::NivelDeCriacao,
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
        if lido.corresponde(
            tipo,
            hipotese,
            especificacao,
            nivel,
            interessados_solicitados,
        ) {
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
            id_tipo: "12".into(),
            tipo: tipo.into(),
            id_hipotese_legal: None,
            especificacao: esp.into(),
            nivel: crate::dominio::NivelDeAcesso::Publico,
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
        nivel: crate::dominio::NivelDeCriacao,
    ) -> Result<Desfecho, ErroConferencia> {
        super::classificar(
            candidatos,
            conferidos,
            ("12", tipo),
            None,
            especificacao,
            nivel,
            &[],
        )
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
                ("12", TIPO),
                None,
                ESPECIFICACAO,
                crate::dominio::NivelDeCriacao::Publico,
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
            classificar(
                &candidatos,
                &conferidos,
                TIPO,
                ESPECIFICACAO,
                crate::dominio::NivelDeCriacao::Publico
            ),
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
            classificar(
                &candidatos,
                &conferidos,
                TIPO,
                ESPECIFICACAO,
                crate::dominio::NivelDeCriacao::Publico
            ),
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
            classificar(
                &candidatos,
                &conferidos,
                TIPO,
                ESPECIFICACAO,
                crate::dominio::NivelDeCriacao::Publico
            ),
            Ok(Desfecho::SemCorrespondenciaObservada)
        );
    }

    #[test]
    fn nenhum_candidato_na_fotografia_nao_prova_nao_aplicacao() {
        assert_eq!(
            classificar(
                &[],
                &[],
                TIPO,
                ESPECIFICACAO,
                crate::dominio::NivelDeCriacao::Publico
            ),
            Ok(Desfecho::SemCorrespondenciaObservada)
        );
    }

    #[test]
    fn candidato_sem_leitura_nao_pode_ser_ignorado() {
        let candidatos = vec![candidato("17", "NUP-17")];
        assert_eq!(
            classificar(
                &candidatos,
                &[],
                TIPO,
                ESPECIFICACAO,
                crate::dominio::NivelDeCriacao::Publico
            ),
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
            classificar(
                &candidatos,
                &conferidos,
                TIPO,
                ESPECIFICACAO,
                crate::dominio::NivelDeCriacao::Publico
            ),
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
                classificar(
                    &candidatos,
                    &[prova_errada],
                    TIPO,
                    ESPECIFICACAO,
                    crate::dominio::NivelDeCriacao::Publico
                ),
                Err(ErroConferencia::IdentidadeDivergente)
            );
        }
    }

    #[test]
    fn identidade_da_hipotese_e_do_tipo_discrimina_homonimos() {
        let candidatos = vec![candidato("17", "NUP-17")];
        let mut lido = prova("17", "NUP-17", TIPO, ESPECIFICACAO);
        lido.nivel = crate::dominio::NivelDeAcesso::Restrito;
        for (tipo, hipotese, coincide) in [
            ("12", Some("31"), true),
            ("13", Some("31"), false),
            ("12", Some("32"), false),
            ("12", None, false),
        ] {
            lido.id_tipo = tipo.into();
            lido.id_hipotese_legal = hipotese.map(str::to_owned);
            let nivel = crate::dominio::NivelDeCriacao::Restrito;
            assert_eq!(
                lido.corresponde(("12", TIPO), Some("31"), ESPECIFICACAO, nivel, &[]),
                coincide
            );
            let desfecho = super::classificar(
                &candidatos,
                std::slice::from_ref(&lido),
                ("12", TIPO),
                Some("31"),
                ESPECIFICACAO,
                nivel,
                &[],
            )
            .unwrap();
            assert_eq!(
                matches!(desfecho, Desfecho::CoincidenciaUnica { .. }),
                coincide
            );
        }
    }

    #[test]
    fn nivel_divergente_nao_comprova_criacao() {
        let candidatos = vec![candidato("17", "NUP-17")];
        let mut prova_restrita = prova("17", "NUP-17", TIPO, ESPECIFICACAO);
        prova_restrita.nivel = crate::dominio::NivelDeAcesso::Restrito;
        assert_eq!(
            classificar(
                &candidatos,
                &[prova_restrita],
                TIPO,
                ESPECIFICACAO,
                crate::dominio::NivelDeCriacao::Publico
            ),
            Ok(Desfecho::SemCorrespondenciaObservada)
        );
    }
}
