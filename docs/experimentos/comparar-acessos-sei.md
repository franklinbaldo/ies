---
type: ExperimentPlan
title: Comparar REST, WebServices e Web no IES
description: Experimento discriminante por operação sem presumir equivalência
status: planned
---

# Comparar rotas de acesso ao SEI

## Questão

Para quais operações a API REST, os WebServices ou o caminho Web
observado conseguem entregar o **mesmo resultado de domínio** com
menor custo, menor fragilidade e sem alterar permissões?

A disponibilidade do módulo, os escopos do serviço, a identidade da
unidade e a semântica do retorno devem ser observados, não adivinhados.

## Contratos localizados

O [OpenAPI REST WSSEI fixado por commit](../interfaces/rest-wssei.md)
possui 127 operações e 34 schemas; o [WSDL/XSD SOAP](../interfaces/soap-sei.md)
é dinâmico por instalação e não foi coletado em ambiente live.
Os contratos não comprovam disponibilidade nem autorização. Algumas
instalações não disponibilizam REST funcional, e o WSDL SOAP pode exigir
credenciais até para ser obtido. Aplicar a
[política de acesso por instalação](../interfaces/acesso-por-instalacao.md).
Nenhum ensaio live é iniciado sem host, identidade e permissão acordados.

## Primeiros pares a investigar

| Operação do IES | Alternativa documentada | Pergunta discriminante |
| --- | --- | --- |
| `unidade.listar_processos` | REST `GET /processo/listar` | Traz recebidos e gerados, com mesmos conjuntos, filtros, estado de leitura e completude? |
| `processo.obter` | REST `GET /processo/{protocolo}` | Aceita ID interno; como resolver NUP visível com segurança e preservar documentos/propriedades? |
| `documento.obter` | REST `GET /documento/listar/{procedimento}` e consultas específicas | Como lidar com conteúdo, tipos, acesso, lote e versões? |
| `usuario.listar_unidades` | REST `GET /usuario/unidades` | O resultado e contexto autorizado são iguais aos oferecidos ao usuário Web? |
| `processo.enviar` | REST `POST /processo/enviar` | Em ambiente de teste autorizado: prévia, destinatários, efeito e conferência são equivalentes? |

Essas rotas existem na especificação do WSSEI em
[commit identificado](https://github.com/pengovbr/mod-wssei/blob/c6bfce09bd181a5f245e9dbb596d5916dba69727/swagger/wssei.json).
**Ainda não são rotas implementadas no IES nem confirmadas nas
sete instalações do acervo.**

## Desenho experimental

1. **Compatibilidade documental:** extrair do Swagger os schemas,
   parâmetros e erros relevantes. Marcar campos sem equivalência
   demonstrada e permissões cuja disponibilidade é desconhecida.
2. **Reprodução Web:** gerar modelos com as cassettes existentes,
   por versão/instalação, sem alterar dados no SEI.
3. **Fixtures REST/SOAP sintéticas:** produzir somente modelos
   inventados para testar parsing e invariantes. Nunca registrar
   resposta autenticada de produção no Git.
4. **Autorização prévia:** obter do administrador confirmação do módulo,
   credenciais adequadas para REST e separadamente para SOAP/WSDL, e a
   identidade/unidade permitidas. Uma negativa de autenticação não é
   prova de ausência da API; não tentar endpoints por enumeração.
5. **Teste controlado**, se houver instalação de laboratório com
   interfaces habilitadas e autorização: mesma identidade de usuário,
   mesma unidade, mesmo conjunto estável de processos, mesmos filtros
   e mesma janela de leitura. Separar diferenças legítimas de
   inconsistências.
6. **Critérios:** comparar conjuntos por ID e NUP, totais e páginas,
   permissões, campos requeridos, erros, round-trips, latência,
   bytes transferidos e custo de manutenção. Ordenação só é critério
   se fizer parte do contrato.
7. **Escritas:** em transações de laboratório descartáveis, observar
   a transição do estado e releitura independente. Um timeout
   é efeito incerto, sem fallback e sem repetição automática.

## Resultado esperado

Para cada operação/instalação registrar em OKF:
`disponibilidade` (confirmada/ausente/desconhecida),
`equivalencia` (passa/falha/não testada),
`requisitos` de autenticação/instalação, `custo` comparável,
`prova` e `próxima ação`. Não preencher estados com suposições.

A arquitetura deve ser escolhida **após** os testes. Se REST não
reproduzir a semântica, manter o caminho Web comprovado para essa
operação. Caso não exista caminho seguro, deixar a operação
indisponível em vez de inventar uma adaptação.
