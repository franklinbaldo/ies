//! A tela do Nível de detalhe do anti-IES reflete somente o SQLite da
//! instalação/cenário. A gravação depende de outra fatia, não há POST fictício.
use crate::{Estado, Resposta, escapar, protocolo_sintetico};
use iesei::colunas_detalhe::{self, Coluna, Configuracao, Erro};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

fn html(configuracao: &Configuracao, gravado: bool) -> String {
    // Apenas o POST persistido sinaliza gravação. O GET é uma leitura.
    let sinal = if gravado {
        "<script>function inicializar(){if ('1'=='1'){window.parent.location='controlador.php?acao=procedimento_controlar';infraFecharJanelaModal();}}</script>"
    } else {
        ""
    };
    let hash = protocolo_sintetico::INFRA_HASH;
    let mut pagina = format!(
        "<html><head><title>SEI - Configurar Detalhe</title>{sinal}</head><body>\
         <form id='frmAcessoExternoCancelar' method='post' onsubmit='return OnSubmitForm();' \
         action='controlador.php?acao=procedimento_configurar_detalhe&amp;acao_origem=procedimento_configurar_detalhe&amp;infra_hash={hash}'>\
         <input type='hidden' id='hdnInfraTipoPagina' name='hdnInfraTipoPagina' value='2'>\
         <button type='submit' id='sbmSalvar' name='sbmSalvar' value='Salvar'>Salvar</button>"
    );
    for opcao in &configuracao.opcoes {
        let campo = opcao.coluna.campo();
        let rotulo = escapar(opcao.coluna.rotulo());
        let checked = if opcao.marcada {
            " checked='checked'"
        } else {
            ""
        };
        write!(
            pagina,
            "<div><input type='checkbox' id='{campo}' name='{campo}' class='infraCheckbox'{checked}>\
             <label for='{campo}'>{rotulo}</label></div>"
        )
        .expect("String");
    }
    pagina.push_str("</form></body></html>");
    pagina
}

/// Consulta ou grava as colunas oferecidas. POST somente com o formulário
/// efetivamente servido; os efeitos persistem no SQLite efêmero.
pub(crate) fn responder(
    metodo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !matches!(metodo, "GET" | "POST") {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let configuracao = match colunas_detalhe::ler(banco, instalacao, cenario) {
        Ok(Some(configuracao)) => configuracao,
        Ok(None) | Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    if metodo == "GET" {
        return Resposta::html(html(&configuracao, false));
    }
    let Some(campos) = crate::processo_envio::pares(corpo) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if crate::processo_envio::campo_unico(&campos, "hdnInfraTipoPagina") != Some("2")
        || crate::processo_envio::campo_unico(&campos, "sbmSalvar") != Some("Salvar")
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let mut marcadas = Vec::<Coluna>::new();
    for (nome, valor) in campos {
        if matches!(nome, "hdnInfraTipoPagina" | "sbmSalvar") {
            continue;
        }
        let Some(coluna) = Coluna::TODAS
            .into_iter()
            .find(|coluna| coluna.campo() == nome)
        else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        if valor != "on"
            || marcadas.contains(&coluna)
            || !configuracao
                .opcoes
                .iter()
                .any(|opcao| opcao.coluna == coluna)
        {
            return Resposta::nova(Estado::RequisicaoInvalida);
        }
        marcadas.push(coluna);
    }
    match colunas_detalhe::aplicar_marcas(banco, instalacao, cenario, &marcadas) {
        Ok(depois) => Resposta::html(html(&depois, true)),
        Err(Erro::DadoInvalido) => Resposta::nova(Estado::RequisicaoInvalida),
        Err(Erro::Sqlite(_)) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iesei::colunas_detalhe::{Coluna, Opcao};
    use iesei::observacao::UnidadeExibida;

    #[test]
    fn form_por_estado_e_nao_por_html_fixo() {
        let config = Configuracao {
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            opcoes: vec![
                Opcao {
                    coluna: Coluna::Atribuicao,
                    marcada: true,
                },
                Opcao {
                    coluna: Coluna::Anotacao,
                    marcada: false,
                },
            ],
        };
        let html = html(&config, false);
        assert!(html.contains("name='chkSinAtribuicao' class='infraCheckbox' checked"));
        assert!(html.contains("name='chkSinAnotacao' class='infraCheckbox'>"));
        assert!(html.contains("id='frmAcessoExternoCancelar'"));
        assert!(!html.contains("name='chkSinMarcadores'"));
    }
}
