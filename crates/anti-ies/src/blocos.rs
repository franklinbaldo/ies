//! Projecao HTML de uma unica pagina de blocos a partir de estado SQLite.
//! A tela nao e um HTML gravado: faixas e linhas sao consultadas a cada GET.
use crate::{Estado, Resposta, escapar, parametro_unico, protocolo_sintetico};
use iesei::blocos::{self, Pagina, Tipo};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

const HASH: &str = protocolo_sintetico::INFRA_HASH;

pub(crate) fn acrescentar_menu(
    mut resposta: Resposta,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if resposta.estado != Estado::Ok || !resposta.corpo.contains("</body>") {
        return resposta;
    }
    let mut links = String::new();
    for tipo in [Tipo::Assinatura, Tipo::Interno, Tipo::Reuniao] {
        match blocos::ler(banco, instalacao, cenario, tipo) {
            Ok(Some(_)) => {
                write!(
                    links,
                    "<a href='controlador.php?acao={}&amp;infra_hash={HASH}'>Blocos {}</a>",
                    tipo.acao(),
                    tipo.nome()
                )
                .expect("escrever em String");
            }
            Ok(None) => {}
            Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
        }
    }
    resposta.corpo = resposta
        .corpo
        .replacen("</body>", &format!("{links}</body>"), 1);
    resposta
}

fn paginador(saida: &mut String, nome: &str, pagina: usize, observadas: &[usize]) {
    write!(saida, "<select name='{nome}'>").expect("String");
    for &numero in observadas {
        // A Infra publica indice zero-based, enquanto a CLI publica 1-based.
        let indice = numero - 1;
        let selecionada = if numero == pagina { " selected" } else { "" };
        write!(
            saida,
            "<option value='{indice}'{selecionada}>{numero}</option>"
        )
        .expect("String");
    }
    saida.push_str("</select>");
}

fn html(pagina: &Pagina, numero_pagina: usize, observadas: &[usize]) -> String {
    let mut saida = format!(
        "<html><body><form id='frmBlocoLista' method='post' action='controlador.php?acao={}&amp;infra_hash={HASH}'>",
        pagina.tipo.acao()
    );
    write!(
        saida,
        "<input type='hidden' name='hdnInfraTipoPagina' value='1'>\
        <input type='hidden' name='hdnInfraPaginaAtual' value='{}'>",
        numero_pagina - 1,
    )
    .expect("String");
    if observadas.len() > 1 {
        paginador(
            &mut saida,
            "selInfraPaginacaoSuperior",
            numero_pagina,
            observadas,
        );
        paginador(
            &mut saida,
            "selInfraPaginacaoInferior",
            numero_pagina,
            observadas,
        );
    }
    if pagina.tipo == Tipo::Assinatura {
        write!(saida, "<button id='btnNovo' type='button' onclick=\"location.href='controlador.php?acao=bloco_assinatura_cadastrar&amp;infra_hash={HASH}'\">Novo</button>").expect("String");
    }
    saida.push_str("<table id='tblBlocos'>");
    if let Some((inicio, fim)) = pagina.faixa {
        write!(
            saida,
            "<caption>Lista de Blocos ({} registros - {inicio} a {fim}):</caption>",
            pagina.total
        )
        .expect("String");
    } else {
        write!(
            saida,
            "<caption>Lista de Blocos ({} registros):</caption>",
            pagina.total
        )
        .expect("String");
    }
    saida.push_str("<tr><th></th><th>Número</th><th>Estado</th><th>Descrição</th></tr>");
    for linha in &pagina.linhas {
        let numero = escapar(&linha.numero);
        write!(saida, "<tr><td><input type='checkbox'></td><td><a href='controlador.php?acao=rel_bloco_protocolo_listar&amp;id_bloco={numero}&amp;infra_hash={HASH}'>{numero}</a></td><td>{}</td><td>{}</td></tr>",
            escapar(&linha.estado), escapar(&linha.descricao)).expect("String");
    }
    saida.push_str("</table></form></body></html>");
    saida
}

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" && metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(tipo) = parametro_unico(alvo, "acao").and_then(Tipo::da_acao) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let numero_pagina = if metodo == "GET" {
        1
    } else {
        let Some(indice) = crate::documentos::campo_formulario_unico(corpo, "hdnInfraPaginaAtual")
            .and_then(|v| v.parse::<usize>().ok())
        else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        let Some(numero_pagina) = indice.checked_add(1) else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        // Nao aceitar um POST apontando simultaneamente para duas paginas.
        for campo in ["selInfraPaginacaoSuperior", "selInfraPaginacaoInferior"] {
            if let Some(valor) = crate::documentos::campo_formulario_unico(corpo, campo)
                && valor.parse::<usize>() != Ok(indice)
            {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
        }
        numero_pagina
    };
    let observadas = blocos::paginas_observadas(banco, instalacao, cenario, tipo);
    let pagina = blocos::ler_pagina(banco, instalacao, cenario, tipo, numero_pagina);
    match (observadas, pagina) {
        (Ok(observadas), Ok(Some(pagina))) if observadas.contains(&numero_pagina) => {
            Resposta::html(html(&pagina, numero_pagina, &observadas))
        }
        _ => Resposta::nova(Estado::DadosInsuficientes),
    }
}

pub(crate) fn cadastro(
    metodo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo == "POST" {
        let Some(pares) = crate::processo_envio::pares(corpo) else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        let campo = |nome| crate::processo_envio::campo_unico(&pares, nome);
        let Some(descricao) =
            campo("txtDescricao").and_then(crate::processo_envio::decodificar_formulario)
        else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        if pares.len() != 3
            || campo("selGrupoBloco") != Some("null")
            || campo("sbmCadastrarBloco") != Some("Salvar")
            || descricao.trim().is_empty()
        {
            return Resposta::nova(Estado::RequisicaoInvalida);
        }
        return match blocos::acrescentar_assinatura(banco, instalacao, cenario, &descricao) {
            Ok(id) => Resposta::redirecionar(format!(
                "controlador.php?acao=bloco_assinatura_listar&id_bloco={id}"
            )),
            Err(_) => Resposta::nova(Estado::DadosInsuficientes),
        };
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    match blocos::ler(banco, instalacao, cenario, Tipo::Assinatura) {
        Ok(Some(_)) => Resposta::html(format!(
            "<html><body><form id='frmBlocoCadastro' method='post' action='controlador.php?acao=bloco_assinatura_cadastrar&amp;infra_hash={HASH}'>\
            <textarea name='txtDescricao'></textarea>\
            <select name='selGrupoBloco'><option value='null'>Nenhum</option></select>\
            <button type='submit' name='sbmCadastrarBloco' value='Salvar'>Salvar</button></form></body></html>"
        )),
        _ => Resposta::nova(Estado::DadosInsuficientes),
    }
}
