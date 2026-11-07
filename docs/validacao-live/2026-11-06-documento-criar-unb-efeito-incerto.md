---
type: Guide
title: Ensaio UnB de documento criar com efeito incerto
description: Prévia, confirmação única e releitura posterior em treinamento em 06/11/2026
---

# Documento criar — ensaio UnB de 06/11/2026

Issue [#246](https://github.com/franklinbaldo/ies/issues/246);
PR [#248](https://github.com/franklinbaldo/ies/pull/248).
**Somente UnB de TREINAMENTO**, conta pública. Não houve uso de produção.
Processo observado: `23106.001868/2026-91`; tipo exato: `Despacho`;
nível pedido: público. A descrição exclusiva da tentativa foi
`Teste IES issue 246 - 2026-11-06 - 7d0ba8f`.

## Sequência ao vivo

[Run 42257050267](https://github.com/franklinbaldo/ies/actions/runs/42257050267),
branch `codex/246-documento-criar`, binário Release compilado no runner.
Arquivos de origem, credenciais e sessão tinham permissões `0600`;
somente campos saneados de resultado foram publicados.

| Etapa | Resultado | Requisições reais observadas |
| --- | --- | --- |
| Login UnB, conta pública | **PASS**, saída 0 | **6** |
| Prévia `ies documento criar`, sem `--confirmar` | **PASS**, saída 0 | **4** |
| Gate de sessão, unidade publicada, processo/tipo/nível e ausência de alteração | **PASS** | 0 adicionais |
| Um único `--confirmar` após pausa de 30 s | **FAIL**, saída **3**, `efeito_incerto_na_criacao_de_documento` | **null — não observado** |
| Releitura pelo próprio comando de criação | Resultado **inconclusivo** (saída 3) | Incluída no total não observado da etapa anterior |
| Repetição do POST | **NÃO EXECUTADA**, proibida | **0** |

O próprio fluxo contém somente uma tentativa de POST criador e uma
releitura após envio. A instrumentação do script não publicou quantas
requisições ocorreram durante a confirmação; **não inferir 6 GET/1 POST
sintéticos como contagem real**, nem declarar que houve aceite. Efeito
incerto não é sucesso, mas também não prova ausência de criação.
A execução remota encerrou após a saída 3, sem tentar escrever novamente.

## Releitura independente posterior, somente leitura

[Run 42257690822](https://github.com/franklinbaldo/ies/actions/runs/42257690822),
sem comando de escrita e sem `--confirmar`, também na UnB de treinamento:

- login **PASS**, **6 requisições**;
- `ies processo 23106.001868/2026-91 --sem-andamentos
  --sem-marcadores --sem-propriedades`: **PASS**, **2 requisições**;
- árvore lida com **quatro documentos** com números SEI:
  `0012129` (id `51710`), `0012130` (id `51711`),
  `0012139` (id `51724`) e **`0012220` (id `51816`,
  rótulo `Despacho 0012220`)**.

**Resíduo possível, NÃO ATRIBUÍDO:** `Despacho 0012220`, id `51816`,
no processo acima. A releitura posterior comprova que o documento está na
árvore naquele momento, **mas não havia fotografia externa anterior da
árvore no log** capaz de atribuí-lo à tentativa desta sessão.
Não afirmar que o IES o criou nem descartá-lo como resíduo. A saída
de criação continua **efeito incerto (3)**. Não executar outro
`--confirmar` neste processo para "corrigir" o ensaio.

## Limites e próximos passos

- Ambos os workflows temporários de ensaio e releitura foram **removidos
  do branch**. Nenhum segredo, cookie, `infra_hash`, URL assinada ou
  corpo da resposta HTTP foi registrado nos logs ou neste relatório.
- A verificação posterior é **PASS de leitura**, não PASS de escrita.
  Para avançar no diagnóstico da confirmação, observar a classe do
  retorno do POST em futura instalação de treinamento, de forma
  saneada e **sem repetir a escrita incerta**. Considerar outro
  processo autorizado apenas com prévia nova e justificativa explícita.
- Melhorar a contagem de requisições na saída de efeito incerto:
  hoje aparece `null`, não zero. Não fabricar contagem a partir
  do teste de transporte injetado.
- A capacidade #246 permanece **WIP**; não encerrar a issue por
  este ensaio. Compilação e replays do head anterior ao workflow:
  [CI 42256098252](https://github.com/franklinbaldo/ies/actions/runs/42256098252)
  **6/6 PASS**; revalidar o head deste relatório no CI.
