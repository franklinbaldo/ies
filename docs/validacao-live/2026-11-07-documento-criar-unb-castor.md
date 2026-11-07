---
type: Guide
title: Criação de documento interno — ensaio UnB 7 de novembro de 2026
description: Confirmação única de Despacho na UnB de treinamento com releitura e identificação do resíduo
---

# Documento criar — validação live UnB (07/11/2026)

Issue [#246](https://github.com/franklinbaldo/ies/issues/246).
**Exclusivamente** `https://sei.treinamento.unb.br/sei`, conta
pública da [lista de treinamento](../interfaces/credenciais-de-treinamento.md).
Sem produção; sessão e credenciais em arquivos privados `0600`,
sem expor cookies, senhas, URLs assinadas ou `infra_hash`.

## Evidência BDD-246-01 e BDD-246-02

| Etapa | Resultado live |
| --- | --- |
| [Prévia independente #42414585368](https://github.com/franklinbaldo/ies/actions/runs/42414585368) | Login PASS **6 requisições**; lista da unidade **92 processos**, 91 candidatos após exclusões; prévia de Despacho público PASS **5 requisições**, `confirmado=false`, `alterado=false`, nenhuma criação |
| [Ciclo único #42414996851](https://github.com/franklinbaldo/ies/actions/runs/42414996851) | Login PASS 6; lista PASS, 92 processos; prévia PASS 5; foto independente inicial `ies processo NUP`: PASS **3 requisições** |
| [Confirmação no mesmo ciclo](https://github.com/franklinbaldo/ies/actions/runs/42414996851) | **Uma única invocação** de `ies documento criar --processo NUP --tipo Despacho --nivel publico --confirmar`: saída **0**, `confirmado=true`, `alterado=true`, `numero_sei=0012225`, **9 requisições** |
| [Verificação externa posterior](https://github.com/franklinbaldo/ies/actions/runs/42414996851) | `ies processo NUP` independente: PASS **3 requisições**, **5 → 6 documentos**, exatamente 1 ID novo; os cinco IDs anteriores preservados |
| [Releitura por número #42415526804](https://github.com/franklinbaldo/ies/actions/runs/42415526804) | Novo login PASS 6; `ies documento 0012225` PASS **5 requisições**; documento Despacho `0012225`, ID `51825`, NUP `23106.001037/2025-39`, processo ID `33892` |

A CLI realizou **uma única invocação** de confirmação, cujo código
contém um único POST criador e releitura verificadora. O ensaio não
instrumentou os métodos HTTP internos: as 9 requisições da confirmação
não significam 9 POSTs. A comparação externa da árvore mostrou um
documento adicional, e a leitura direta confirmou número, tipo,
ID e processo. Tipo de processo observado:
`Pessoal: Licença Capacitação Profissional`.
Não houve retry nem segunda confirmação.

## Limites, exclusões e registros honestos

O processo `23106.001868/2026-91` da UnB permanece excluído por
[efeito incerto anterior](2026-11-06-documento-criar-unb-efeito-incerto.md).
Um ensaio de prévia anterior nesta rodada para
`23106.001592/2026-41` teve saída 1 depois de login PASS, mas a
falha do roteiro de diagnóstico impediu registrar o subcódigo:
[run #42414242464](https://github.com/franklinbaldo/ies/actions/runs/42414242464).
Esse pedido não foi repetido.

Os workflows temporários foram excluídos da branch após disparo;
o único código de produção usado foi o existente na base
`87f9d9e` da `main` (Release compilado com sucesso no runner).
**RED novo não se aplica**, porque a rodada foi de validação live,
sem mudar comportamento ou reescrever testes verdes.
Esta prova vale **somente para a UnB**: outras instalações, níveis
restrito e sigiloso ainda precisam de validação própria.

**FEITO:** BDD-246-01 e BDD-246-02 demonstrados live por prévia,
confirmação única e releitura independente; resíduo registrado.

**CLI:** `ies documento criar --processo 23106.001037/2025-39 --tipo Despacho --nivel publico --confirmar`
retornou `numero_sei=0012225`. **Não repetir a criação nesse processo.**

**CONTINUAR:** validar outra instalação apenas em TREINAMENTO com
novo NUP e prévia própria; preservar o resíduo já gerado. Issue aberta;
Castor não faz merge.
