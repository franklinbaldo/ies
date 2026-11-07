//! Projeção de fatos já extraídos pelos leitores do Controle do IES.
use iesei::instalacao::Instalacao;
use iesei::observacao::{
    Cobertura, FaixaObservada, FormaDoControle, ObservacaoDoControle, ProcessoNaLista,
    Proveniencia, TabelaObservada, UnidadeExibida,
};

fn intervalo(v: Option<(usize, usize)>) -> Option<Option<FaixaObservada>> {
    match v {
        Some((a, b)) => Some(Some(FaixaObservada::nova(a, b)?)),
        None => Some(None),
    }
}

fn tabela(
    t: &crate::dominio::Tabela<crate::dominio::Processo>,
    faixa: Option<(usize, usize)>,
) -> Option<Cobertura<TabelaObservada<ProcessoNaLista>>> {
    let linhas = t
        .processos
        .iter()
        .map(|p| ProcessoNaLista::novo(p.id.as_str(), &p.numero))
        .collect::<Option<Vec<_>>>()?;
    Some(Cobertura::Observada(
        TabelaObservada::nova(t.total_declarado, intervalo(faixa)?, linhas).ok()?,
    ))
}

/// Retorna somente fatos que o parser de produção reconheceu. Nunca faz HTTP.
pub(super) fn da_tela(
    html: &str,
    instalacao: Instalacao,
) -> Option<ObservacaoDoControle<ProcessoNaLista>> {
    let (unidade, forma) = if crate::controle_reconhecido::publica_visao_detalhada(html) {
        let leitura = crate::caixa_detalhada::ler_paginas(&[html.to_owned()], None).ok()?;
        let linhas = leitura
            .processos
            .iter()
            .map(|p| ProcessoNaLista::novo(p.id.as_str(), &p.numero))
            .collect::<Option<Vec<_>>>()?;
        let tabela = TabelaObservada::nova(
            Some(leitura.total_declarado),
            intervalo(leitura.faixa_observada)?,
            linhas,
        )
        .ok()?;
        (
            leitura.unidade,
            FormaDoControle::Detalhada {
                processos: Cobertura::Observada(tabela),
            },
        )
    } else {
        let leitura = crate::paginas::juntar(&[html.to_owned()], None).ok()?;
        let recibidos = tabela(
            &leitura.caixa.recebidos,
            leitura.recebidos.faixa_observada(),
        )?;
        let gerados = tabela(&leitura.caixa.gerados, leitura.gerados.faixa_observada())?;
        (
            leitura.unidade?,
            FormaDoControle::Resumida {
                recebidos: recibidos,
                gerados,
            },
        )
    };
    Some(ObservacaoDoControle {
        instalacao,
        unidade: UnidadeExibida::nova(&unidade)?,
        forma,
        proveniencia: Proveniencia::LeituraWeb {
            operacao: "unidade.atual".to_owned(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controle_vazio_e_observacao_suficiente() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let instalacao = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
        let lida = da_tela(html, instalacao).unwrap();
        assert_eq!(lida.unidade.as_str(), "PPI");
        assert!(lida.suficiente_para_servir());
    }

    #[test]
    fn pagina_sem_controle_nao_e_observacao() {
        let instalacao = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
        assert!(da_tela("<p>login</p>", instalacao).is_none());
    }
}
