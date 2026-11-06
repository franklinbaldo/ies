"""Audita pontos estáticos de emissão de erros Rust (issue #299).

Uso (prévia, não altera código nem é gate):
  uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 \\
      python scripts/auditar_emissoes_erro.py [--json]

Cada referência de construção a Enum::Variante conta como ponto de emissão.
Padrões de match, testes, uses e aliases não resolvidos NÃO são inferidos.
A análise é conservadora: serve para planejar um codemod por domínio,
não para renomear automaticamente variantes só para satisfazer uma métrica.

Observação: o código reutiliza as rotinas AST de migrar_erro_html_plano;
isso evita escrever mais um parser parcial de Rust.
"""
from __future__ import annotations

import argparse
from collections import defaultdict
import json

from migrar_erro_html_edicoes import contexto_de_padrao
from migrar_erro_html_plano import (
    Resolvedor, em_teste, enums_com_causa, ler_fontes, mesmo, nos, texto,
)

FORMAS_HTML = {"ausente", "repetido", "valor", "html", "atributo"}


def em_use(no):
    pai = no.parent
    while pai is not None:
        if pai.type == "use_declaration":
            return True
        if pai.type in ("function_item", "impl_item", "mod_item"):
            return False
        pai = pai.parent
    return False


def construcoes():
    fontes = ler_fontes()
    enums = enums_com_causa(fontes)
    resolvedor = Resolvedor(fontes, enums)
    emissao = defaultdict(list)
    html = defaultdict(list)

    for arquivo, (fonte, arvore) in fontes.items():
        nome_arquivo = str(arquivo.relative_to(arquivo.parents[1]))
        # Os pontos criados em código de teste não são novas origens em produção.
        for no in nos(arvore.root_node, "scoped_identifier"):
            pai = no.parent
            if (pai is not None and pai.type == "scoped_identifier"
                    and mesmo(pai.child_by_field_name("path"), no)):
                continue
            if em_teste(no, fonte, arquivo) or em_use(no) or contexto_de_padrao(no):
                continue
            caminho = no.child_by_field_name("path")
            nome = no.child_by_field_name("name")
            if caminho is None or nome is None:
                continue
            prefixo, variante = texto(caminho, fonte), texto(nome, fonte)
            dono = resolvedor.nome(arquivo, no, prefixo)
            if dono in enums and variante in enums[dono].variantes:
                emissao[(dono, variante)].append({
                    "arquivo": nome_arquivo, "linha": no.start_point.row + 1,
                    "referencia": texto(no, fonte),
                })

            # O ErroHtml contém o local de falha como texto, fora do sistema de tipos.
            # É um inventário separado: não fingir que 'onde' é variante de enum.
            if prefixo.split("::")[-1] == "ErroHtml" and variante in FORMAS_HTML:
                if pai is None or pai.type != "call_expression":
                    continue
                args = pai.child_by_field_name("arguments")
                if args is None or not args.named_children:
                    continue
                primeiro = args.named_children[0]
                if primeiro.type != "string_literal":
                    continue
                html[texto(primeiro, fonte)].append({
                    "arquivo": nome_arquivo, "linha": no.start_point.row + 1,
                    "construtor": variante,
                })

    return emissao, html, len(enums)


def resumo(emissao, html, enum_count):
    repetidas = []
    for (dono, variante), locais in sorted(emissao.items()):
        unicos = {(p["arquivo"], p["linha"]) for p in locais}
        if len(unicos) > 1:
            repetidas.append({
                "erro": f"{dono}::{variante}",
                "pontos": len(unicos),
                "locais": locais,
            })
    repetidas.sort(key=lambda x: (-x["pontos"], x["erro"]))
    duplicadas_html = [
        {"onde": nome, "pontos": len({(p["arquivo"], p["linha"]) for p in locais}),
         "locais": locais}
        for nome, locais in html.items()
        if len({(p["arquivo"], p["linha"]) for p in locais}) > 1
    ]
    duplicadas_html.sort(key=lambda x: (-x["pontos"], x["onde"]))
    return {
        "enums_univocos_com_derive": enum_count,
        "variantes_referenciadas": len(emissao),
        "variantes_com_varios_pontos": len(repetidas),
        "construcoes_html_com_onde": sum(len(locais) for locais in html.values()),
        "identificadores_html_repetidos": len(duplicadas_html),
        "reutilizacoes": repetidas,
        "reutilizacoes_html": duplicadas_html,
        "limitacoes": [
            "Apenas enums com derive Causa, nome não ambíguo; não cobre todos os enums.",
            "Referências em macro_invocation/token_tree e variantes importadas sem qualificador não são resolvidas.",
            "Ocorrências em teste e padrão de match são excluídas.",
            "Construção num mesmo ponto executado várias vezes conta uma única origem estática.",
            "A análise não prova que duas ocorrências têm a mesma semântica nem que substituí-las é seguro.",
        ],
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true", help="emitir inventário JSON")
    args = ap.parse_args()
    erros, html, total = construcoes()
    resultado = resumo(erros, html, total)
    if args.json:
        print(json.dumps(resultado, ensure_ascii=False, indent=2))
        return
    print(
        f"Enums resolvidos: {total}; variantes referenciadas: "
        f"{resultado['variantes_referenciadas']}; "
        f"variantes com >1 ponto de emissão: {resultado['variantes_com_varios_pontos']}"
    )
    print(
        f"Construções ErroHtml: {resultado['construcoes_html_com_onde']}; "
        f"identificadores 'onde' repetidos: {resultado['identificadores_html_repetidos']}"
    )
    for item in resultado["reutilizacoes"][:50]:
        pontos = ", ".join(f"{x['arquivo']}:{x['linha']}" for x in item["locais"][:7])
        print(f"  {item['erro']}: {item['pontos']} pontos — {pontos}")
    for item in resultado["reutilizacoes_html"][:15]:
        pontos = ", ".join(f"{x['arquivo']}:{x['linha']}" for x in item["locais"][:5])
        print(f"  ErroHtml onde={item['onde']}: {item['pontos']} pontos — {pontos}")
    print("PRÉVIA apenas; cobertura parcial descrita em --json.")


if __name__ == "__main__":
    main()
