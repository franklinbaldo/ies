//! Uma resposta `procedimento_visualizar`: a mesma leitura de
//! `ler_arvore_da_pagina` (árvore, vínculo, movidos e os três links da
//! página) ou, com `abrir_pastas=1` na URL servida, a de
//! `aberta_inteira_da_pagina`; depois pastas, situação e natureza.
use super::leitores::Cassette;
use super::{Leitor, Registro, Resultado};
use crate::arvore_processo::{AbertoEm, ErroArvore, MotivoAbertoEm, ler_acao_de_abrir_pastas};
use crate::cassette::acervo::gravacao::{self, Resposta};
use crate::processo_obter::abertura::{ArvoreAberta, ler_arvore_da_pagina};
use crate::processo_obter::pastas::aberta_inteira_da_pagina;

pub(super) fn ler(c: &Cassette<'_>, r: &Resposta<'_>, html: &str, id: &str) -> Vec<Registro> {
    // Árvore sem nó `PROCESSO` não declara NUP: o leitor diz por quê.
    let nup = c.nup(id).unwrap_or("");
    let abrir = gravacao::parametro(r.interacao, "abrir_pastas") == ["1"];
    let lida = if abrir {
        aberta_inteira_da_pagina(&c.instalacao, nup, html, id)
    } else {
        ler_arvore_da_pagina(&c.instalacao, nup, html, id)
    };
    let aberta = match lida {
        Ok(aberta) => aberta,
        Err(erro) => {
            let mut registros = vec![c.registro(
                Leitor::Arvore,
                r.passo,
                Resultado::Recusado(format!("{erro:?}")),
            )];
            if abrir {
                let recusa = Resultado::Recusado(format!("{erro:?}"));
                registros.push(c.registro(Leitor::Pastas, r.passo, recusa));
            }
            return registros;
        }
    };
    let mut registros = vec![arvore(c, r, &aberta)];
    registros.extend(pastas(c, r, &aberta, html, abrir));
    registros.push(situacao(c, r, &aberta.arvore.aberto_em));
    registros.push(natureza(c, r, &aberta));
    registros.extend(links(c, r, &aberta));
    registros
}

fn arvore(c: &Cassette<'_>, r: &Resposta<'_>, aberta: &ArvoreAberta) -> Registro {
    let a = &aberta.arvore;
    let mut registro = c.registro(Leitor::Arvore, r.passo, Resultado::Lido);
    let documentos = a.documentos.len() as u64;
    let pastas = a.pastas.len() as u64;
    let anexados = a.anexados.len() as u64;
    let movidos = a.documentos_movidos.len() as u64;
    registro.contagens = vec![
        ("nos", 1 + documentos + pastas + anexados + movidos),
        ("documentos", documentos),
        ("pastas", pastas),
        ("anexados", anexados),
        ("movidos", movidos),
        ("aguarde", u64::from(a.pastas_nao_carregadas)),
        ("anexado_a", u64::from(a.anexado_a.is_some())),
        (
            "movidos_com_destino",
            a.documentos_movidos
                .iter()
                .filter(|m| m.destino.is_some())
                .count() as u64,
        ),
    ];
    for movido in &a.documentos_movidos {
        registro
            .formas
            .push(if movido.presente_entre_os_documentos {
                "movido_presente_entre_os_documentos".into()
            } else {
                "movido_ausente_dos_documentos".into()
            });
    }
    registro
}

fn pastas(
    c: &Cassette<'_>,
    r: &Resposta<'_>,
    aberta: &ArvoreAberta,
    html: &str,
    abrir: bool,
) -> Option<Registro> {
    let a = &aberta.arvore;
    let mut registro = c.registro(Leitor::Pastas, r.passo, Resultado::Lido);
    registro.contagens = vec![("pastas", a.pastas.len() as u64)];
    if abrir {
        registro.formas.push("aberta_por_abrir_pastas".into());
    } else if a.pastas_nao_carregadas {
        match ler_acao_de_abrir_pastas(&c.instalacao, html, &a.processo.id) {
            Ok(Some(_)) => registro.formas.push("fechada_com_acao_de_abrir".into()),
            Ok(None) => {
                let recusa = format!("{:?}", ErroArvore::PastasNaoCarregadasSemAcaoDeAbrir);
                registro.resultado = Resultado::Recusado(recusa);
            }
            Err(erro) => {
                registro.resultado = Resultado::Recusado(format!("{erro:?}"));
            }
        }
    } else if a.pastas.is_empty() {
        return None;
    } else {
        registro.formas.push("carregadas".into());
    }
    for pasta in &a.pastas {
        if let Some(verificacao) = pasta.verificacao {
            registro.formas.push(verificacao.como_texto().into());
        }
    }
    Some(registro)
}

fn situacao(c: &Cassette<'_>, r: &Resposta<'_>, aberto_em: &AbertoEm) -> Registro {
    let (resultado, forma) = match aberto_em {
        AbertoEm::Unidades(unidades) => {
            let atribuidas = unidades.iter().filter(|u| u.atribuido_a.is_some()).count();
            let forma = match (unidades.len(), atribuidas) {
                (0, _) => "nenhum_andamento_aberto".to_owned(),
                (1, 0) => "uma_unidade".to_owned(),
                (1, _) => "uma_unidade_com_atribuicao".to_owned(),
                (_, 0) => "varias_unidades".to_owned(),
                _ => "varias_unidades_com_atribuicao".to_owned(),
            };
            (Resultado::Lido, Some(forma))
        }
        AbertoEm::Sobrestado(unidades) => {
            let forma = if unidades.len() == 1 {
                "sobrestado_numa_unidade"
            } else {
                "sobrestado_em_varias_unidades"
            };
            (Resultado::Lido, Some(forma.to_owned()))
        }
        AbertoEm::NaoLido(MotivoAbertoEm::HtmlDaRaizForaDaFormaObservada) => (
            Resultado::Recusado(
                MotivoAbertoEm::HtmlDaRaizForaDaFormaObservada
                    .como_texto()
                    .to_owned(),
            ),
            None,
        ),
        AbertoEm::NaoLido(motivo) => (
            Resultado::NaoAplicavel(motivo.como_texto().to_owned()),
            None,
        ),
    };
    let mut registro = c.registro(Leitor::Situacao, r.passo, resultado);
    registro.formas.extend(forma);
    registro
}

fn natureza(c: &Cassette<'_>, r: &Resposta<'_>, aberta: &ArvoreAberta) -> Registro {
    let documentos = &aberta.arvore.documentos;
    let mut registro = c.registro(Leitor::Natureza, r.passo, Resultado::Lido);
    let conta = |f: &dyn Fn(&crate::arvore_processo::NaturezaDoDocumento) -> bool| {
        documentos.iter().filter(|d| f(&d.natureza)).count() as u64
    };
    registro.contagens = vec![
        ("documentos", documentos.len() as u64),
        ("sem_icone", conta(&|n| n.icone.is_none())),
        ("cancelado_nao_lido", conta(&|n| n.cancelado.is_none())),
        ("cancelados", conta(&|n| n.cancelado == Some(true))),
        ("com_dica", conta(&|n| n.dica.is_some())),
    ];
    for documento in documentos {
        let icone = documento.natureza.icone.as_deref().unwrap_or("(não lido)");
        registro.formas.push(format!("icone:{icone}"));
    }
    if registro.contagem("sem_icone") > 0 {
        registro.resultado = Resultado::Recusado("icone_nao_lido".to_owned());
    }
    registro
}

fn links(c: &Cassette<'_>, r: &Resposta<'_>, aberta: &ArvoreAberta) -> [Registro; 3] {
    let historico = match &aberta.historico {
        Ok(_) => Resultado::Lido,
        Err(crate::historico_processo::ErroLinkDoHistorico::LinkDoHistoricoAusenteNaArvore) => {
            Resultado::NaoAplicavel(format!(
                "{:?}",
                crate::historico_processo::ErroLinkDoHistorico::LinkDoHistoricoAusenteNaArvore
            ))
        }
        Err(erro) => Resultado::Recusado(format!("{erro:?}")),
    };
    let propriedades = match &aberta.propriedades {
        Ok(Some(_)) => Resultado::Lido,
        Ok(None) => {
            Resultado::NaoAplicavel("tela_de_propriedades_nao_publicada_na_barra".to_owned())
        }
        Err(erro) => Resultado::Recusado(format!("{erro:?}")),
    };
    let marcadores = match &aberta.marcadores {
        Ok(Some(_)) => Resultado::Lido,
        Ok(None) => Resultado::NaoAplicavel("tela_de_marcadores_nao_publicada_na_barra".to_owned()),
        Err(erro) => Resultado::Recusado(format!("{erro:?}")),
    };
    [
        c.registro(Leitor::LinkDoHistorico, r.passo, historico),
        c.registro(Leitor::LinkDasPropriedades, r.passo, propriedades),
        c.registro(Leitor::LinkDosMarcadores, r.passo, marcadores),
    ]
}
