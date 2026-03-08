# Plano Geral da CLI - Inflation Data Management

## **TL;DR**
CLI interativa baseada em menu para gerenciar taxas de inflação anuais. Dados armazenados em `data/inflation/rates.json` como array JSON. Suporta criar, listar, editar e deletar entradas com validações, confirmação apenas para deletar, e feedback simples (✓/✗).

---

## **Estrutura de Arquivos**
```
src/
├── main.rs              (entry point do binário)
└── cli/                 (módulo isolado da lib)
    ├── mod.rs           (re-exports públicos)
    ├── menu.rs          (sistema menu interativo)
    ├── operations.rs    (CRUD: create, list, edit, delete)
    ├── storage.rs       (persistência JSON)
    └── validation.rs    (validações de entrada)
```

---

## **Operações Implementadas**

| Operação | Entrada | Processamento | Saída |
|----------|---------|---------------|-------|
| **1. Criar** | Ano + Taxa | Valida duplicata, ordem, precisão | ✓ Sucesso / ✗ Erro |
| **2. Listar** | - | Carrega dados | Tabela (Ano \| Taxa) |
| **3. Editar** | Ano + Nova Taxa | Valida precisão | ✓ Atualizado / ✗ Erro |
| **4. Deletar** | Ano | Pede confirmação (S/N) | ✓ Deletado / Cancelado |
| **Q. Sair** | - | Encerra | - |

---

## **Características do MVP**

✅ **Scope**: Inflation Data Management only  
✅ **Storage**: Arquivo JSON simples (`[]`)  
✅ **Auto-load**: Cria `data/inflation/` automaticamente  
✅ **Validações**:
- Ano deve ser único (não permite duplicatas)
- Taxa positiva com **máximo 2 casas decimais**
- Anos em ordem ascendente (validado ao salvar)

✅ **UX**:
- Menu interativo com prompt "Deseja fazer outra operação? (S/N)"
- Tabela formatada (Ano | Taxa %) para listagem
- Apenas deletar requer confirmação
- Mensagens simples: `✓ Sucesso` / `✗ Erro`

✅ **Fluxo**: Menu → Operação → Prompt Continuar → Loop/Sair

---

## **Dados Persistidos**

**Arquivo**: `data/inflation/rates.json`

```json
[
  { "year": 2020, "inflation": "1.50" },
  { "year": 2021, "inflation": "2.25" },
  { "year": 2022, "inflation": "0.99" }
]
```

---

## **Decisões Arquiteturais**

| Decisão | Valor | Motivo |
|---------|-------|--------|
| **CLI Scope** | Inflation Data | MVP focado, budget para Fase 2 |
| **Storage Format** | Array JSON | Simples, sem metadados |
| **Validação de Ordem** | On Save | Permite entrada flexível |
| **Confirmações** | Apenas Delete | Delete é destrutivo; Edit é reversível |
| **Precisão** | 2 casas decimais | Padrão financeiro |
| **Decoupling** | CLI ≠ LIB | CLI é cliente standalone da lib |

---

## **Comparativo: Plano Original vs Implementação Final** 

| Aspecto | Plano Original | Implementação Final |
|--------|---|---|
| Precisão de Taxa | 4 casas decimais | **2 casas decimais** ✓ |
| CLI na LIB? | Sim (pub mod cli) | **Não, standalone** ✓ |
| Estrutura de Dados | JSON array | **Mantido** ✓ |
| Operações CRUD | Create, List, Edit, Delete | **Todas implementadas** ✓ |
| Validações | Sim | **Implementadas + reforçadas** ✓ |
| Confirmação Delete | Sim | **Implementada** ✓ |
| Menu Interativo | Sim | **Implementado** ✓ |

---

## **Próximas Fases (Roadmap)**

**Fase 2**: Budget Management CLI  
**Fase 3**: Portfolio Transformations  
**Fase 4**: Relatórios comparativos  

---

## **Status de Implementação**

- ✅ Estrutura de projeto criada
- ✅ Storage layer implementado
- ✅ Validation layer implementado
- ✅ Menu system implementado
- ✅ CRUD operations implementadas
- ✅ CLI desacoplado da lib
- ✅ Testes passando
- ✅ Precisão ajustada para 2 casas decimais
